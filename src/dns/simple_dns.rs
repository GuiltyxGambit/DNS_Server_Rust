
pub struct DNS_Server {
    pub word: String
}

impl DNS_Server {
    pub fn new(word: &str) -> DNS_Server { 
        // Implimentation:
        DNS_Server {word: word.to_string()}
    }
}

