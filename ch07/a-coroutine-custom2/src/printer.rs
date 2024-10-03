pub struct Printer {
    last: Option<String>,
    n: usize,
}

impl Printer {
    fn new() -> Self {
        Self { last: None, n: 0 }
    }

    fn println(&mut self, msg: String) {
        match &self.last {
            Some(l) if l == &msg => {
                let n = self.n.to_string();
                let width = n.len();
                let msg = "\x08".repeat(width);
                print!("{msg}{n}");
                // print!("{n}");
            }
            Some(l) => self.n = 0,
            None => (),
        }
        self.last = Some(msg);
        self.n += 1;
    }
}
