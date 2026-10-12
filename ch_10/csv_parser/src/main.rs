use std::{collections::HashMap, hash::Hash};

#[derive(Debug)]
struct CsvParser<'a> {
    input: &'a str,
    pos: usize, //current position
    headers: Vec<&'a str>,
}

impl<'a> CsvParser<'a> {
    //to find headers
    //1. find the cur position of `/n`
    //if pos - 1 == /r split until pos-1
    //else split until pos
    fn new(input: &'a str) -> CsvParser<'a> {
        let pos = input
            .find('\n')
            .map(|p| {
                if p > 0 && input.as_bytes()[p - 1] == b'\r' {
                    p - 1
                } else {
                    p
                }
            })
            .unwrap_or(input.len()); //unwrap or in case theres only one line = headers 

        let headers = if input.is_empty() {
            Vec::new()
        } else {
            input[0..pos].split(',').collect()
        };

        //prolly should write extra conditions to verify if str.len == pos then dont inc pos
        // pos += 1;

        //doing this again cos need to be at the last index,
        //in case of crlf, index will be at \n if it weren't gor this
        //
        let mut pos = input.find('\n').unwrap_or(input.len()) + 1;

        //incase of empty lines after headers point position to the first
        //non empty line
        loop {
            let rest = &input[pos..];

            if let Some(r) = rest.strip_prefix("\r\n") {
                pos = input.len() - r.len();
            } else if let Some(r) = rest.strip_prefix('\n') {
                pos = input.len() - r.len();
            } else {
                break;
            }
        }

        CsvParser {
            input,
            pos, //store the end of headers
            headers,
        }
    }

    fn headers(&self) -> &[&'a str] {
        &self.headers
    }

    //read spec for impl
    fn record(&self, row: Vec<&'a str>) -> HashMap<&'a str, &'a str> {
        let mut rec = HashMap::new();

        let smaller_len = if self.headers.len() > row.len() {
            row.len()
        } else {
            self.headers.len()
        };

        for i in 0..smaller_len {
            rec.insert(self.headers[i], row[i]);
        }

        rec
    }
}

impl<'a> Iterator for CsvParser<'a> {
    type Item = Vec<&'a str>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.input.len() {
            return None;
        }

        let cur_pos = self.pos;
        //this pos is relativge to the subslice
        //position of the last element in the row
        let mut pos = self.input[cur_pos..]
            .find('\n')
            .unwrap_or(self.input[cur_pos..].len());

        //when pos = 0, without this check, we get a
        //subrtraction overflow error (pos - 1)
        let mut row_end;
        if pos > 0 {
            row_end = match self.input.as_bytes()[cur_pos..][pos - 1] {
                b'\r' => pos - 1,
                _ => pos,
            };
        } else {
            row_end = pos;
        }

        //make the indices relative to self.input and not the subslice
        pos += cur_pos;
        row_end += cur_pos;

        //incase of empty lines
        loop {
            let rest = &self.input[pos..];

            if let Some(r) = rest.strip_prefix("\r\n") {
                pos = self.input.len() - r.len();
            } else if let Some(r) = rest.strip_prefix('\n') {
                pos = self.input.len() - r.len();
            } else {
                break;
            }
        }

        self.pos = pos; //next line starts after \n

        Some(self.input[cur_pos..row_end].split(',').collect())
    }
}

fn query<'a>(csv: &'a str, column: &str, value: &str) -> Vec<HashMap<&'a str, &'a str>> {
    todo!();
}

fn main() {
    let mut a = CsvParser::new("a,b\r\n\r\n1,2\r\n\r\n");
    dbg!(&a);
    dbg!(&a.next());
    dbg!(&a.next());
}

// fn rows(input: &str) -> Vec<Vec<&str>> {
//     CsvParser::new(input).collect()
// }

#[cfg(test)]
mod tests;
