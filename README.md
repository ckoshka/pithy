## pithy 0.1.0 - an absurdly fast, strangely accurate, summariser
```
Quick example:
pithy -f your_file_here.txt --sentences 4
```

### You can play with it either:
* As a crate that comes with both a library and a CLI tool: https://crates.io/crates/pithy
* Or on the web demo: https://pithy.glossandra.repl.co/

![speedtest](https://user-images.githubusercontent.com/48640397/152093217-cc6efabf-1ebb-46a1-b463-004ca6680f22.gif)

![final_demo_gif_2](https://user-images.githubusercontent.com/48640397/152093010-dd8667c6-f0e6-4a37-8c32-c2f99e993cf1.gif)

--**help:**

    Print this help message
    
-**f:**

    The file pithy will read from. Required.

--**sentences:**

    The number of sentences for pithy to return. Defaults to 3.
    

--**density**

    Experimental setting. Defaults to 3. Setting it lower 
    makes for more general summaries with more common words,
    setting it higher prioritises important highlights that
    might not be central to the text.

--**by_section:**

    If set, pithy splits the text into sections, and each section is
    summarized separately. Defaults to false.

--**chunk_size:**

    The number of sentences to read at a time. Defaults to 500 
    if unspecified.

--**force_all:**

    If set, pithy reads the text all at once. Can be quite 
    slow once you go past the 7k mark. Defaults to false.

--**force_chunk:**

    If set, regardless of how large the text is, pithy splits it
    into chunks. Should be used in combination with chunk_size 
    and by_section.
--ngrams:
    If set, pithy uses ngrams rather than words. 
    It's usually crap, but you might use it as a last resort 
    for non-spaced languages that you can't pre-tokenise. 
    Defaults to false.

--**min_length:**

    The minimum sentence length before filtering. Defaults to 30.

--**max_length:**

    The maximum sentence length before filtering. Defaults to 1500.

--**separator:**

    The separator used to split the text into sentences. 
    Defaults to '. '. You can type newline to separate by newlines.

--**clean_whitespace:**

    If set, removes sentences with excessive whitespace. Useful for 
    pdfs and copy-pastes from websites.

--**clean_nonalphabetic:**

    If set, removes sentences with too many non-alphabetic characters.

--**clean_caps:**

    If set, removes sentences with too many capital letters. Useful 
    if the text contains a lot of references or indices.

--**length_penalty**

    The length penalty. Defaults to 1.5. Decrease to make glance for longer 
    sentences, increase for shorter sentences.

--**no_context**

    If set, the context surrounding sentences isn't provided. 
    Defaults to false.

--**relevance**

    If set, the sentences are sorted by their relevance rather 
    than their order in the original text. Defaults to false.

--**nobar**

    If set, the progress bar is not printed. Defaults to false because
    progress bars are cool.
