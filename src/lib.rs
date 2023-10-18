#[derive(Debug, PartialEq, Clone)]
pub enum JsonValue {
    Bool(bool),
    Number(f64),
    Null,
    Array(Vec<JsonValue>),
    String(String),
    Object(Vec<(String, JsonValue)>),
}

#[derive(Debug)]
pub enum JsonError {
    InvalidFormat(String),
    EndOfStr,
}

pub struct Json {
    pos: usize,
    json: String,
}

pub trait JsonParser {
    fn parse(&mut self) -> Result<JsonValue, JsonError>;
    fn parse_bool(&mut self) -> Result<JsonValue, JsonError>;
    fn parse_number(&mut self) -> Result<JsonValue, JsonError>;
    fn parse_null(&mut self) -> Result<JsonValue, JsonError>;
    fn parse_array(&mut self) -> Result<JsonValue, JsonError>;
    fn parse_string(&mut self) -> Result<JsonValue, JsonError>;
    fn parse_object(&mut self) -> Result<JsonValue, JsonError>;
    fn parse_key(&mut self) -> String;
}

impl Json {
    pub fn new(json: String) -> Self {
        Self { pos: 0, json }
    }

    fn consume_char(&mut self) {
        self.pos += 1;
    }

    fn consume_chars(&mut self, n: usize) {
        self.pos += n
    }

    fn current_char(&self) -> Option<char> {
        self.json.chars().nth(self.pos)
    }

    fn next_char(&self) -> Option<char> {
        self.json.chars().nth(self.pos + 1)
    }

    fn look_ahead(&self, length: usize) -> String {
        self.json
            .chars()
            .into_iter()
            .skip(self.pos + 1)
            .into_iter()
            .take(length)
            .collect()
    }

    fn invalid_format(&self, invalid: char) -> JsonError {
        JsonError::InvalidFormat(format!(
            "Invalid format at pos {} found: '{}'",
            self.pos, invalid
        ))
    }
}

impl JsonParser for Json {
    fn parse(&mut self) -> Result<JsonValue, JsonError> {
        loop {
            match self.current_char() {
                Some(' ') | Some('\n') | Some(':') | Some(',') | Some('\t') => self.consume_char(),
                Some(_) | None => break,
            };
        }

        match self.current_char() {
            Some('{') => self.parse_object(),
            Some('T') | Some('F') | Some('t') | Some('f') => self.parse_bool(),
            Some('-') | Some('0') | Some('1') | Some('2') | Some('3') | Some('4') | Some('5')
            | Some('6') | Some('7') | Some('8') | Some('9') => self.parse_number(),
            Some('N') | Some('n') => self.parse_null(),
            Some('[') => self.parse_array(),
            Some('"') => self.parse_string(),
            Some(c) => Err(self.invalid_format(c)),
            None => Err(JsonError::EndOfStr),
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, JsonError> {
        match self.current_char() {
            Some('t') | Some('T') => match self.look_ahead(3).to_lowercase().as_str() {
                "rue" => {
                    self.consume_chars(4);
                    Ok(JsonValue::Bool(true))
                }
                c => Err(JsonError::InvalidFormat(format!(
                    "Invalid boolean at {} found char: '{}'",
                    self.pos, c
                ))),
            },
            Some('f') | Some('F') => match self.look_ahead(4).to_lowercase().as_str() {
                "alse" => {
                    self.consume_chars(5);
                    Ok(JsonValue::Bool(false))
                }
                c => Err(JsonError::InvalidFormat(format!(
                    "Invalid boolean at {} found char: '{}'",
                    self.pos, c
                ))),
            },
            None => Err(JsonError::EndOfStr),
            c => Err(JsonError::InvalidFormat(format!(
                "Invalid boolean at {} found char: '{:?}'",
                self.pos, c
            ))),
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, JsonError> {
        let num: String = self
            .json
            .chars()
            .into_iter()
            .skip(self.pos)
            .into_iter()
            .take_while(|c| c.is_digit(10) || *c == '.' || *c == '-')
            .collect::<String>();

        match num.parse::<f64>() {
            Ok(v) => {
                self.consume_chars(num.len());
                Ok(JsonValue::Number(v))
            }
            Err(e) => Err(JsonError::InvalidFormat(format!(
                "Invalid number format at {} '{}'",
                self.pos, e
            ))),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, JsonError> {
        match self.look_ahead(3).as_str() {
            "ull" => {
                self.consume_chars(4);
                Ok(JsonValue::Null)
            }
            c => Err(JsonError::InvalidFormat(format!(
                "Invalid null value at {} '{}'",
                self.pos, c
            ))),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, JsonError> {
        let mut array = Vec::new();

        loop {
            self.consume_char();

            match self.current_char() {
                Some(',') | Some(' ') => {
                    self.consume_char();
                    continue;
                }
                Some(']') => {
                    self.consume_char();
                    break;
                }
                Some(_) => {
                    let value = self.parse().unwrap();
                    array.push(value);
                }
                None => break,
            }
        }
        Ok(JsonValue::Array(array))
    }

    fn parse_string(&mut self) -> Result<JsonValue, JsonError> {
        self.consume_char();
        let str: String = self
            .json
            .chars()
            .into_iter()
            .skip(self.pos)
            .into_iter()
            .take_while(|c| *c != '"')
            .collect::<String>();

        self.consume_chars(str.len());

        Ok(JsonValue::String(str))
    }

    fn parse_key(&mut self) -> String {
        let str = self
            .json
            .chars()
            .into_iter()
            .skip(self.pos)
            .into_iter()
            .take_while(|c| *c != '"')
            .collect::<String>();

        self.consume_chars(str.len());

        str
    }

    fn parse_object(&mut self) -> Result<JsonValue, JsonError> {
        let mut obj = Vec::new();
        loop {
            self.consume_char();
            match self.current_char() {
                Some('}') => {
                    self.consume_char();
                    break;
                }
                // Fattar inte .. dum i huvudet
                Some(',') | Some(' ') | Some(':') | Some('\n') | Some('\t') => {
                    self.consume_char();
                    continue;
                }
                Some(_) => {
                    let key = self.parse_key();
                    self.consume_char();
                    let value = self.parse().unwrap();
                    obj.push((key, value));
                    continue;
                }
                None => break,
            };
        }
        Ok(JsonValue::Object(obj))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{Json, JsonParser, JsonValue};

    #[test]
    fn parse_object_num() {
        let data = r#"{"age":4}"#.to_string();

        let mut obj = Vec::new();
        obj.push(("age".to_string(), JsonValue::Number(4.0)));

        let mut json = Json::new(data);
        assert_eq!(json.parse().unwrap(), JsonValue::Object(obj));
    }

    #[test]
    fn parse_null() {
        let data = "null".to_string();
        let mut json = Json::new(data);
        assert_eq!(json.parse_null().unwrap(), JsonValue::Null);
    }

    #[test]
    fn parse_knull() {
        let datar2 = "nkll".to_string();
        let mut json2 = Json::new(datar2);
        assert!(json2.parse_null().is_err());
    }

    #[test]
    fn parse_num() {
        let data = "1234".to_string();
        let mut json = Json::new(data);
        assert_eq!(json.parse_number().unwrap(), JsonValue::Number(1234.0));
    }

    #[test]
    fn parse_num_single() {
        let data = "2".to_string();
        let mut json = Json::new(data);
        assert_eq!(json.parse_number().unwrap(), JsonValue::Number(2.0));
    }

    #[test]
    fn parse_num_negative() {
        let data = "-122".to_string();
        let mut json = Json::new(data);
        assert_eq!(json.parse_number().unwrap(), JsonValue::Number(-122.0));
    }

    #[test]
    fn parse_num_fraction() {
        let data = "12.34".to_string();
        let mut json = Json::new(data);
        assert_eq!(json.parse_number().unwrap(), JsonValue::Number(12.34));
    }

    #[test]
    fn parse_num_leading_zero() {
        let data = "02".to_string();
        let mut json = Json::new(data);
        assert_eq!(json.parse_number().unwrap(), JsonValue::Number(2.0));
    }

    #[test]
    fn parse_bool() {
        let mut cases = HashMap::new();
        cases.insert("true", JsonValue::Bool(true));
        cases.insert("false", JsonValue::Bool(false));
        cases.insert("FALSE", JsonValue::Bool(false));
        cases.insert("TrUe", JsonValue::Bool(true));

        for key in cases.keys() {
            cases.get(*key);
            let mut json = Json::new(key.to_string());
            assert_eq!(cases.get(*key).unwrap(), &json.parse().unwrap());
        }
    }

    #[test]
    fn parse_array() {
        let data = "[1,2,3]".to_string();

        let mut json = Json::new(data);
        assert_eq!(
            json.parse_array().unwrap(),
            JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::Number(2.0),
                JsonValue::Number(3.0)
            ])
        )
    }

    #[test]
    fn parse_array_bool() {
        let data = "[true,false,false]".to_string();

        let mut json = Json::new(data);
        assert_eq!(
            json.parse_array().unwrap(),
            JsonValue::Array(vec![
                JsonValue::Bool(true),
                JsonValue::Bool(false),
                JsonValue::Bool(false)
            ])
        )
    }

    #[test]
    fn parse_string() {
        let data = "\"apa\"".to_string();

        let mut json = Json::new(data);
        assert_eq!(
            json.parse_string().unwrap(),
            JsonValue::String("apa".to_string())
        )
    }

    #[test]
    fn look_ahed() {
        let testdata = "12345";
        let json = Json::new(testdata.to_string());

        assert_eq!(json.look_ahead(3), String::from("234"));
        assert_eq!(json.look_ahead(10), String::from("2345"));
    }
}
