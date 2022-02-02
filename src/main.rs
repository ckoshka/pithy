use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
//Imports Arc:
use std::sync::{Arc, Mutex};

#[derive(Debug, PartialEq, Clone)]
pub struct Sentence {
    pub index: usize,
    pub length: usize,
    pub outgoing_connections: Option<HashMap<usize, usize>>, //first represents sentence index, second represents the number of outgoing connections
    pub text: String,
    pub words: HashSet<String>,
    pub number_of_connections: f32,
}

#[derive(Debug, Clone)]
pub struct Summariser<'a> {
    pub sentences: HashMap<usize, Sentence>,
    matrix: Option<Vec<Vec<f32>>>,
    bias_list: HashSet<String>,
    bias_strength: Option<f32>,
    progress_bar_off: bool,
    _marker: std::marker::PhantomData<&'a str>,
}

pub fn jaccard_similarity(set_a: &HashSet<String>, set_b: &HashSet<String>) -> f32 {
    let intersection_of: HashSet<String> =
        set_a.intersection(&set_b).map(|x| x.to_string()).collect();
    let union_of: HashSet<String> = set_a.union(&set_b).map(|x| x.to_string()).collect();
    let jaccard_similarity = intersection_of.len() as f32 / union_of.len() as f32;
    jaccard_similarity
}

use indicatif::ProgressBar;

impl<'a> Summariser<'a> {
    pub fn from_raw_text(
        raw_text: String,
        separator: &str,
        min_length: usize,
        max_length: usize,
        ngrams: bool,
        progress_bar: bool,
        bias_strength: Option<f32>,
    ) -> Summariser<'a> {
        let sentences: Arc<Mutex<HashMap<usize, Sentence>>> = Arc::new(Mutex::new(HashMap::new()));
        //let tokenizer = get_tokenizer_from_text(raw_text.clone());
        //Split the text into chunks of 100 words each
        //let chunked_text = raw_text
        //.split(|c: char| !c.is_alphanumeric() && c != ' ')
        //.map(|x| x.to_string())
        //.collect::<Vec<String>>()
        //.chunks(35)
        //.map(|x| x.join(" "))
        //.collect::<Vec<String>>();
        let all_sentences = raw_text.split(separator).collect::<Vec<&str>>();
        //let number_of_sentences_as_u32 = all_sentences.len() as u32;
        //let bar = ProgressBar::new(number_of_sentences_as_u32 as u64);
        for (i, sentence) in all_sentences.iter().enumerate() {
            if sentence.len() > min_length && sentence.len() < max_length {
                //println!("{}", i);
                let mut words: HashSet<String> = HashSet::new();
                if !ngrams {
                    words = HashSet::from_iter(
                        sentence.split_whitespace().map(|word| word.to_string()),
                    );
                } else {
                    for n in 7..15 {
                        let ngrams = sentence
                            .chars()
                            .collect::<Vec<char>>()
                            .windows(n)
                            .map(|x| x.iter().collect::<String>())
                            .collect::<Vec<String>>();
                        words.extend(ngrams);
                    }
                }
                let outgoing_connections = HashMap::new();
                let sentence = Sentence {
                    index: i,
                    length: sentence.len(),
                    outgoing_connections: Some(outgoing_connections),
                    text: sentence.to_string(),
                    words: words,
                    number_of_connections: 0.0,
                };
                sentences.lock().unwrap().insert(i, sentence.clone());
                //}
            }
        }
        let final_sentences = sentences.lock().unwrap().clone();
        //bar.finish();
        //println!("{:?}", final_sentences);
        Summariser {
            sentences: final_sentences,
            matrix: None,
            bias_list: HashSet::new(),
            bias_strength: bias_strength,
            progress_bar_off: progress_bar,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn retrieve_sentence_by_index(&self, index: usize) -> Option<Sentence> {
        self.sentences.get(&index).cloned()
    }

    pub fn from_sentences(sentences: Vec<String>) -> Summariser<'a> {
        let converted_and_filtered_sentences = sentences
            .iter()
            .map(|sentence| {
                let words: HashSet<String> = HashSet::from_iter(
                    sentence
                        .to_lowercase()
                        .split_whitespace()
                        .map(|word| word.to_string()),
                );
                let outgoing_connections = HashMap::new();
                let sentence = Sentence {
                    index: 0,
                    length: sentence.len(),
                    outgoing_connections: Some(outgoing_connections),
                    text: sentence.to_string(),
                    words: words,
                    number_of_connections: 0.0,
                };
                sentence
            })
            .collect::<Vec<Sentence>>();
        let mut sentences_map = HashMap::new();
        for (i, sentence) in converted_and_filtered_sentences.iter().enumerate() {
            sentences_map.insert(i, sentence.clone());
        }
        Summariser {
            sentences: sentences_map,
            matrix: None,
            bias_list: HashSet::new(),
            bias_strength: None,
            progress_bar_off: false,
            _marker: std::marker::PhantomData,
        }
    }

    fn white_space_closure_filter(&self, sentence: Sentence) -> bool {
        let whitespace_count = sentence
            .text
            .chars()
            .filter(|&c| c == ' ' || c == '\t' || c == '\n')
            .count();
        let whitespace_percentage = whitespace_count as f32 / sentence.length as f32;
        whitespace_percentage < 0.15
    }

    fn punctuation_closure_filter(&self, sentence: Sentence) -> bool {
        let punc_vec = vec![
            '.', ',', '!', '?', ':', ';', '-', '\'', '"', '[', ']', '(', ')', '{', '}', '<', '>',
            '=', '+', '*', '&', '^', '%', '$', '#', '@', '~', '`', '|', '\\', '/', '_', '0', '1',
            '2', '3', '4', '5', '6', '7', '8', '9',
        ];
        let punctuation_vec: HashSet<&char> = HashSet::from_iter(punc_vec.iter());
        let punctuation_count = sentence
            .text
            .chars()
            .filter(|&c| punctuation_vec.contains(&c))
            .count();
        let punctuation_percentage = punctuation_count as f32 / sentence.length as f32;
        punctuation_percentage < 0.12
    }

    fn caps_closure_filter(&self, sentence: Sentence) -> bool {
        let caps_count = sentence.text.chars().filter(|&c| c.is_uppercase()).count();
        let caps_percentage = caps_count as f32 / sentence.length as f32;
        caps_percentage < 0.02
    }

    //fn too_similar_to_existing_filter(&self, sentence: Sentence) -> bool {
    //let mut similar_to_existing = false;
    //for (k, existing_sentence) in self.sentences.iter() {
    //if k != &sentence.index {
    //let jaccard_similarity =
    //jaccard_similarity(&sentence.words, &existing_sentence.words);
    //if jaccard_similarity > 0.5 {
    //similar_to_existing = true;
    //break;
    //}
    //}
    //}
    //similar_to_existing
    //}

    pub fn clean_sentences(
        &mut self,
        excessive_whitespace: bool,
        excessive_punctuation_and_nums: bool,
        excessive_caps: bool,
    ) -> &mut Summariser<'a> {
        let new_sentences = self
            .sentences
            .clone()
            .into_iter()
            .filter(|(_, sentence)| {
                if excessive_whitespace {
                    self.white_space_closure_filter(sentence.clone())
                } else {
                    true
                }
            })
            .filter(|(_, sentence)| {
                if excessive_punctuation_and_nums {
                    self.punctuation_closure_filter(sentence.clone())
                } else {
                    true
                }
            })
            .filter(|(_, sentence)| {
                if excessive_caps {
                    self.caps_closure_filter(sentence.clone())
                } else {
                    true
                }
            })
            .collect::<HashMap<_, _>>();
        self.sentences = new_sentences;
        self
    }

    fn from_sentences_direct(sentences: HashMap<usize, Sentence>) -> Summariser<'a> {
        Summariser {
            sentences: sentences,
            matrix: None,
            bias_list: HashSet::new(),
            bias_strength: None,
            progress_bar_off: false,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn top_sentences(
        &mut self,
        number_of_sentences_to_return: usize,
        return_summaries_for_each: bool,
        chunk_size: Option<usize>,
        force_sum_all: bool,
        length_penalty: f32,
        force_chunk: bool,
        density: f32,
        bias_list: Option<HashSet<String>>,
        bias_strength: Option<f32>,
        progress_bar: bool,
    ) -> Vec<Sentence> {
        //If longer than 10,000, then divide it into portions of 5000 each. Instantiate new Summarisers and call Summariser::from_sentences_direct on each one, passing in the portion of the original sentences (convert the HashMap to a vec). Then call Summariser::top_sentences on each one, passing in the number of sentences to return. Collect the sentences, and pass them to a new instance of Summariser::from_sentences_direct. Then call Summariser::top_sentences on that instance, passing in the number of sentences to return. Return the result.
        if bias_list.is_some() {
            self.bias_list = bias_list.clone().unwrap();
        }
        if bias_strength.is_some() {
            self.bias_strength = bias_strength.clone();
        } else {
            self.bias_strength = Some(2.0);
        }
        let length_of_sentences = self.sentences.len();
        if !force_sum_all && length_of_sentences > 2000
            || !force_sum_all && return_summaries_for_each
            || force_chunk
        {
            //if chunk_size is specified, then use that. otherwise use a default value of 2000
            let final_chunk_size = match chunk_size {
                Some(chunk_size) => chunk_size,
                None => 500,
            };
            let mut summarisers = self
                .sentences
                .clone()
                .into_iter()
                .collect::<Vec<(usize, Sentence)>>()
                .chunks(final_chunk_size.clone())
                .map(|chunk| {
                    let mut initial = 0;
                    let mut new_sentences = HashMap::new();
                    for (_, sentence) in chunk {
                        new_sentences.insert(initial.clone(), sentence.clone());
                        initial += 1;
                    }
                    Summariser::from_sentences_direct(new_sentences)
                })
                .collect::<Vec<Summariser<'a>>>();
            //println!("Number of summarisers: {}", summarisers.len());
            //let number_of_summarisers = summarisers.len();
            let collected_sentences = summarisers
                .par_iter_mut()
                .map(|summariser| {
                    let indiv_num_to_return = match return_summaries_for_each {
                        true => number_of_sentences_to_return,
                        false => 100, //number_of_sentences_to_return * number_of_summarisers.clone(),
                    };
                    summariser.top_sentences(
                        indiv_num_to_return,
                        false,
                        None,
                        true,
                        length_penalty,
                        false,
                        density,
                        bias_list.clone(),
                        bias_strength,
                        progress_bar,
                    )
                })
                .collect::<Vec<Vec<Sentence>>>();
            if return_summaries_for_each {
                let collected_sentences = collected_sentences
                    .into_iter()
                    .flatten()
                    .collect::<Vec<Sentence>>();
                return collected_sentences;
            } else {
                let collected_sentences = collected_sentences
                    .into_iter()
                    .flatten()
                    .collect::<Vec<Sentence>>();
                let mut summariser = Summariser::from_sentences_direct(
                    collected_sentences
                        .into_iter()
                        .enumerate()
                        .map(|(index, sentence)| (index, sentence))
                        .collect::<HashMap<_, _>>(),
                );
                let final_sentences = summariser.top_sentences(
                    number_of_sentences_to_return,
                    false,
                    None,
                    false,
                    length_penalty,
                    false,
                    density,
                    bias_list.clone(),
                    bias_strength,
                    progress_bar,
                );
                return final_sentences;
            }
        }
        //let word_density_scaleup = 2.0 + (length_penalty - 0.61);
        //One possible implementation:
        //for i in 0..length_of_sentences {
        //for j in i+1..length_of_sentences.clone() {
        //let number_of_connections = self.number_of_word_connections(i.clone(), j.clone());
        //May need to divide no. of connections by length of respective sentence.
        //self.sentences.get_mut(&i).unwrap().outgoing_connections.as_mut().unwrap().insert(j.clone(), number_of_connections);
        //self.sentences.get_mut(&j).unwrap().outgoing_connections.as_mut().unwrap().insert(i.clone(), number_of_connections);
        //}
        //}
        //However, we can use rayon to do this in parallel, and leave the insertions until later
        //Now, instead of modifying the Sentence structs themselves, we can simply fill in an array/matrix of the number of connections between each sentence.
        //Then, we can use the matrix to fill in the outgoing_connections HashMap of each Sentence.
        //The width of the matrix will need to be the value of the maximum key index in self.sentences
        //let max_key_index = self.sentences.keys().max().unwrap();
        //let min_key_index = self.sentences.keys().min().unwrap();
        let mut matrix = vec![vec![0.0; length_of_sentences.clone()]; length_of_sentences.clone()];
        let length_of_sentences_as_u32 = length_of_sentences as u32;
        let bar = ProgressBar::new(length_of_sentences_as_u32 as u64);
        matrix.par_iter_mut().enumerate().for_each(|(i, row)| {
            for j in i + 1..length_of_sentences {
                if let Some(sentence) = self.sentences.get(&i.clone()) {
                    row[j] = (self.number_of_word_connections(i.clone(), j.clone()) as f32)
                        .powf(density)
                        / (sentence.length as f32).powf(length_penalty); //1.1
                }
            }
            if progress_bar {
                bar.inc(1);
            }
        });
        if progress_bar {
            bar.finish();
        }
        self.matrix = Some(matrix.clone());
        //We could update the outgoing_connections HashMap of each Sentence. But that's slow. It's faster to just get the summed values of each row, and sort them from highest to lowest, get the indices of the top n sentences, then the text for those sentences.
        let mut top_sentences = matrix
            .iter()
            .enumerate()
            .map(|(i, row)| (row.iter().sum::<f32>(), i))
            //Filter out any that summed to zero
            .filter(|(sum, _)| *sum > 0.0)
            .collect::<Vec<(f32, usize)>>();
        top_sentences.par_sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        let top_sentences_indices = top_sentences
            .iter()
            .take(number_of_sentences_to_return)
            .map(|x| x.1)
            .collect::<Vec<usize>>()
            .iter()
            .filter(|x| self.sentences.contains_key(x))
            .map(|x| self.sentences.get(x).unwrap().clone())
            .collect::<Vec<Sentence>>();
        //number_of_sentences_to_return = std::cmp::min(number_of_sentences_to_return, top_sentences_indices.len());
        top_sentences_indices
    }
    //Could create a sparse matrix here
    pub fn number_of_word_connections(
        &'a self,
        sentence_a_indx: usize,
        sentence_b_indx: usize,
    ) -> f32 {
        //We could simply call intersection here on the two HashSets of the .words fields for the two sentences
        //let sentence_a = self.sentences.get(&sentence_a_indx).unwrap();
        //let sentence_b = self.sentences.get(&sentence_b_indx).unwrap();
        //However, the index might not exist in the HashMap, so we need to check for that
        //THIS IS THE BUG, IT NEEDS TO BE 0
        //let mut overlapping_words_with_b_length = 1;
        if let Some(sentence_a) = self.sentences.get(&sentence_a_indx) {
            if let Some(sentence_b) = self.sentences.get(&sentence_b_indx) {
                let intersection_length = sentence_a
                    .words
                    .intersection(&sentence_b.words)
                    .collect::<Vec<_>>()
                    .len();
                //.map(|x| x.to_string())
                //if self.bias_list.len() > 0 {
                //overlapping_words_with_b_length = self.bias_list
                //.intersection(&sentence_b.words)
                //.collect::<Vec<_>>()
                //.len() //+ self.bias_list
                //.intersection(&sentence_a.words)
                //.collect::<Vec<_>>()
                //.len();
                //}
                return intersection_length as f32; // * (1.0 + ((overlapping_words_with_b_length as f32 * 3.0).powf(self.bias_strength.unwrap()) / (sentence_a.length as f32).powf(0.64)));
            } else {
                return 0.0;
            }
        } else {
            return 0.0;
        }
    }
}
//TODO: Add flags for sentence cleaning

use colored::*;
//Import Instant
use std::time::Instant;

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    //--bias:
    //slash (i.e \"/\") separated list of words to bias the summary towards.
    //Very experimental. Try lots of synonyms.
    let help_documentation = "
        ------------
        --help:
            Print this help message
        -f:
            The file pithy will read from. Required.
        --sentences:
            The number of sentences for pithy to return. Defaults to 3.
        --bias_strength:
            The strength of the bias, must be an integer. Defaults to 6.
        --by_section:
            If set, pithy splits the text into sections, and each section is
            summarized separately. Defaults to false.
        --chunk_size:
            The number of sentences to read at a time. Defaults to 500 
            if unspecified.
        --force_all:
            If set, pithy reads the text all at once. Can be quite 
            slow once you go past the 7k mark. Defaults to false.
        --force_chunk:
            If set, regardless of how large the text is, pithy splits it
            into chunks. Should be used in combination with chunk_size 
            and by_section.
        --ngrams:
            If set, pithy uses ngrams rather than words. 
            It's usually crap, but you might use it as a last resort 
            for non-spaced languages that you can't pre-tokenise. 
            Defaults to false.
        --min_length:
            The minimum sentence length before filtering. Defaults to 30.
        --max_length:
            The maximum sentence length before filtering. Defaults to 1500.
        --separator:
            The separator used to split the text into sentences. 
            Defaults to '. '. You can type newline to separate by newlines.
        --clean_whitespace:
            If set, removes sentences with excessive whitespace. Useful for 
            pdfs and copy-pastes from websites.
        --clean_nonalphabetic:
            If set, removes sentences with too many non-alphabetic characters.
        --clean_caps:
            If set, removes sentences with too many capital letters. Useful 
            if the text contains a lot of references or indices.
        --length_penalty:
            The length penalty. Defaults to 1.5. Decrease to make glance for longer 
            sentences, increase for shorter sentences.
        --density:
            Experimental setting. Defaults to 3. Setting it lower 
            seems to bias pithy's summaries towards more common words, 
            setting it higher seems to bias summaries towards rarer 
            but more informative words.
        --no_context:
            If set, the context surrounding sentences isn't provided. 
            Defaults to false.
        --relevance:
            If set, the sentences are sorted by their relevance rather 
            than their order in the original text. Defaults to false.
        --nobar:
            If set, the progress bar is not printed. Defaults to false because
            progress bars are cool.
        ------------
        pithy 0.1.0 - an absurdly fast, strangely accurate, summariser
        ------------
        Quick example:
        pithy -f your_file_here.txt --sentences 4
        ";
    if arguments.contains(&"--help".to_string()) || arguments.len() == 1 {
        println!("{}", help_documentation);
        return;
    }
    let by_section = arguments.contains(&"--by_section".to_string());
    let chunk_size = if arguments.contains(&"--chunk_size".to_string()) {
        Some(
            arguments
                .get(arguments.iter().position(|x| x == "--chunk_size").unwrap() + 1)
                .expect("No chunk size provided")
                .parse::<usize>()
                .unwrap(),
        )
    } else {
        None
    };
    let bias_list = if arguments.contains(&"--bias".to_string()) {
        Some(
            arguments
                .get(arguments.iter().position(|x| x == "--bias").unwrap() + 1)
                .expect("No bias list provided")
                .split("/")
                .map(|x| x.to_string())
                .collect::<HashSet<String>>(),
        )
    } else {
        None
    };
    let bias_strength = if arguments.contains(&"--bias_strength".to_string()) {
        Some(
            arguments
                .get(
                    arguments
                        .iter()
                        .position(|x| x == "--bias_strength")
                        .unwrap()
                        + 1,
                )
                .expect("No bias strength provided")
                .parse::<f32>()
                .unwrap(),
        )
    } else {
        Some(2.0)
    };
    let filename = arguments
        .get(arguments.iter().position(|x| x == "-f").unwrap() + 1)
        .expect("No filename provided");
    let number_of_sentences_to_return = if arguments.contains(&"--sentences".to_string()) {
        arguments
            .get(arguments.iter().position(|x| x == "--sentences").unwrap() + 1)
            .expect("No number of sentences provided")
            .parse::<usize>()
            .unwrap()
    } else {
        3
    };
    let force_all = arguments.contains(&"--force_all".to_string());
    let force_chunk = arguments.contains(&"--force_chunk".to_string());
    //if space, we set "." as the separator. if newline, we set "\n" as the separator.
    let separator = if arguments.contains(&"--separator".to_string()) {
        let arg = arguments
            .get(arguments.iter().position(|x| x == "--separator").unwrap() + 1)
            .expect("No separator provided");
        if arg == "newline" {
            "\n"
        } else {
            arg
        }
    } else {
        "."
    };
    let ngrams = arguments.contains(&"--ngrams".to_string());
    let min_length = if arguments.contains(&"--min_length".to_string()) {
        arguments
            .get(arguments.iter().position(|x| x == "--min_length").unwrap() + 1)
            .expect("No minimum length provided")
            .parse::<usize>()
            .unwrap()
    } else {
        50
    };
    let max_length = if arguments.contains(&"--max_length".to_string()) {
        arguments
            .get(arguments.iter().position(|x| x == "--max_length").unwrap() + 1)
            .expect("No maximum length provided")
            .parse::<usize>()
            .unwrap()
    } else {
        1500
    };
    let relevance = arguments.contains(&"--relevance".to_string());
    let no_context = arguments.contains(&"--no_context".to_string());
    let clean_whitespace = arguments.contains(&"--clean_whitespace".to_string());
    let clean_nonalphabetic = arguments.contains(&"--clean_nonalphabetic".to_string());
    let clean_caps = arguments.contains(&"--clean_caps".to_string());
    let length_penalty = if arguments.contains(&"--length_penalty".to_string()) {
        arguments
            .get(
                arguments
                    .iter()
                    .position(|x| x == "--length_penalty")
                    .unwrap()
                    + 1,
            )
            .expect("No length penalty provided")
            .parse::<f32>()
            .unwrap()
    } else {
        0.9
    };
    let density = if arguments.contains(&"--density".to_string()) {
        arguments
            .get(arguments.iter().position(|x| x == "--density").unwrap() + 1)
            .expect("No density provided")
            .parse::<f32>()
            .unwrap()
    } else {
        3.7
    };

    let use_bar = arguments.contains(&"--nobar".to_string());
    let now = Instant::now();
    let raw_text = std::fs::read_to_string(filename).expect("Could not open the file");
    //api: pub fn from_raw_text(raw_text: String, separator: &str, min_length: usize, max_length: usize, ngrams: bool)
    let mut summariser = Summariser::from_raw_text(
        raw_text.clone(),
        separator,
        min_length,
        max_length,
        ngrams,
        !use_bar,
        bias_strength,
    );
    //api: excessive_whitespace: bool, excessive_punctuation_and_nums: bool, excessive_caps: bool,
    if clean_whitespace || clean_nonalphabetic || clean_caps {
        summariser.clean_sentences(clean_whitespace, clean_nonalphabetic, clean_caps);
    }
    //summariser.clean_sentences(clean_whitespace, clean_nonalphabetic, clean_caps);
    //api: number_of_sentences_to_return: usize, return_summaries_for_each: bool, chunk_size: Option<usize>, force_sum_all: bool, length_penalty: f32
    let mut summary = summariser.top_sentences(
        number_of_sentences_to_return,
        by_section,
        chunk_size,
        force_all,
        length_penalty,
        force_chunk,
        density,
        bias_list,
        bias_strength,
        !use_bar,
    );
    if !use_bar {
        println!("Summarising took {} seconds", now.elapsed().as_secs_f32());
    }
    //sort sentences by .index
    if !relevance {
        summary.par_sort_unstable_by(|a, b| a.index.partial_cmp(&b.index).unwrap());
    }
    //The summary is an array of strings, so we'll pretty-print it:
    // /retrieve_sentence_by_index
    let mut sentence_indices = summariser.sentences.keys().cloned().collect::<Vec<usize>>();
    sentence_indices.sort_unstable();
    //summariser.semirandom_walk(summary[0].index, 5);
    for sentence in summary.clone() {
        let index_number = sentence.index;
        //There might be missing indices, so in retrieving the previous sentence, we need to find what the closest number preceding it is
        if !no_context {
            let previous_sentence_indx = sentence_indices
                .iter()
                .filter(|x| **x < index_number)
                .last()
                .unwrap_or(&0);
            let next_sentence_indx = sentence_indices
                .iter()
                .filter(|x| **x > index_number)
                .next()
                .unwrap_or(&0);
            let previous_sentence = if summariser.sentences.get(previous_sentence_indx).is_some() {
                summariser
                    .sentences
                    .get(previous_sentence_indx)
                    .unwrap()
                    .clone()
                    .text
            } else {
                String::from("")
            };
            let next_sentence = if summariser.sentences.get(next_sentence_indx).is_some() {
                summariser
                    .sentences
                    .get(next_sentence_indx)
                    .unwrap()
                    .clone()
                    .text
            } else {
                String::from("")
            };
            print!(
                "\n{}\n{}{}{}{}{}{}",
                sentence.index,
                separator,
                previous_sentence.italic(),
                separator,
                sentence.text.bold().red(),
                separator,
                next_sentence.italic()
            );
        } else {
            println!(
                "{}",
                sentence.index.to_string().underline().italic().magenta()
            );
            println!("{}", sentence.text.bold().cyan());
            println!("")
        }
    }
    println!("")
    //If the bar is turned off, then concatenate sentence.text and write it to stdout (so that the script can be used in pipes)
    //if use_bar {
    //let stdout = std::io::stdout();
    //let lock = stdout.lock();
    //let mut w = std::io::BufWriter::new(lock);
    //let mut output = String::new();
    //for sentence in summary {
    //output.push_str(&sentence.text);
    //output.push_str(&separator);
    // }
    //w.write_all(output.as_bytes()).unwrap();
    //}
}
//Able to take command-line text as input to allow for repeated piping? chunk sizes.
