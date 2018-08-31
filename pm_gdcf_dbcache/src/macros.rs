macro_rules! ident {
    ($ts_iter: ident) => {
        match $ts_iter.next() {
            Some(TokenTree::Ident(ident)) => ident,
            Some(crap) => panic!("Expected ident, got {:?}", crap),
            None => panic!("Excepted ident, got end-of-stream"),
        }
    };
}

macro_rules! joint_punct {
    ($ts_iter: ident, $punct: expr) => {
        match $ts_iter.next() {
            Some(TokenTree::Punct(punct)) =>
                match punct.spacing() {
                    Spacing::Joint if punct.as_char() == $punct => punct,
                    Spacing::Joint => panic!("Expected '{}', got '{}'", $punct, punct.as_char()),
                    _ => panic!("Expected joint '{}', got alone '{}'", $punct, punct.as_char()),
                },
            Some(crap) => panic!("Expected '{}', got {:?}", $punct, crap),
            None => panic!("Excepted '{}', got end-of-stream", $punct),
        }
    };
}

macro_rules! alone_punct {
    ($ts_iter: ident, $punct: expr) => {
        match $ts_iter.next() {
            Some(TokenTree::Punct(punct)) =>
                match punct.spacing() {
                    Spacing::Alone if punct.as_char() == $punct => punct,
                    Spacing::Alone => panic!("Expected '{}', got '{}'", $punct, punct.as_char()),
                    _ => panic!("Expected alone '{}', got joint '{}'", $punct, punct.as_char()),
                },
            Some(crap) => panic!("Expected '{}', got {:?}", $punct, crap),
            None => panic!("Excepted '{}', got end-of-stream", $punct),
        }
    };
}

macro_rules! any_group {
    ($ts_iter: ident) => {
        match $ts_iter.next() {
            Some(TokenTree::Group(group)) => group,
            Some(crap) => panic!("Expected group, got {:?}", crap),
            None => panic!("Excepted group, got end-of-stream"),
        }
    };
}

macro_rules! stream {
    ($($t: tt)*) => {
        {
            let v: Vec<TokenStream> = vec![$($t)*];
            let stream: TokenStream = v.into_iter().collect();
            stream
        }
    };
}
