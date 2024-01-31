#[derive(Debug, PartialEq, Clone)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

pub mod tokenizer {
    use std::{fmt::Display, iter::Peekable, str::Chars};

    use super::Location;

    #[derive(Debug, PartialEq, Clone)]
    pub enum TokenValue {
        Identifier(String),
        Number(f64),
        String(String),
        LeftBrace,
        RightBrace,
        LeftBracket,
        RightBracket,
        LeftParen,
        RightParen,
        Comma,
        Colon,
        Eof,
    }

    impl TokenValue {
        pub fn is_eq_variant(&self, other: &Self) -> bool {
            std::mem::discriminant(self) == std::mem::discriminant(other)
        }
    }

    impl Display for TokenValue {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                TokenValue::Identifier(s) => write!(f, "'{}'", s),
                TokenValue::Number(n) => write!(f, "'{}'", n),
                TokenValue::String(s) => write!(f, "'{}'", s),
                TokenValue::LeftBrace => write!(f, "'{{'"),
                TokenValue::RightBrace => write!(f, "'}}'"),
                TokenValue::LeftBracket => write!(f, "'['"),
                TokenValue::RightBracket => write!(f, "']'"),
                TokenValue::LeftParen => write!(f, "'('"),
                TokenValue::RightParen => write!(f, "')'"),
                TokenValue::Comma => write!(f, "','"),
                TokenValue::Colon => write!(f, "':'"),
                TokenValue::Eof => write!(f, "EOF"),
            }
        }
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct Token {
        pub value: TokenValue,
        pub location: Location,
    }

    impl Token {
        pub fn new(value: TokenValue, location: Location) -> Token {
            Token { value, location }
        }
    }

    #[derive(PartialEq, Debug)]
    pub struct ParserError {
        pub message: String,
        pub location: Option<Location>,
    }

    impl ParserError {
        pub fn new(message: &str, location: &Location) -> ParserError {
            ParserError {
                message: message.to_string(),
                location: Some(location.clone()),
            }
        }
        pub fn without_location(message: &str) -> ParserError {
            ParserError {
                message: message.to_string(),
                location: None,
            }
        }
    }

    struct CharsWithLocation<'a> {
        chars: Peekable<Chars<'a>>,
        location: Location,
    }

    impl<'a> CharsWithLocation<'a> {
        pub fn new(input: &str) -> CharsWithLocation {
            CharsWithLocation {
                chars: input.chars().peekable(),
                location: Location { line: 1, column: 1 },
            }
        }

        pub fn peek(&mut self) -> Option<&char> {
            self.chars.peek()
        }

        pub fn next(&mut self) -> Option<char> {
            match self.chars.next() {
                Some(c) => {
                    match c {
                        '\n' => {
                            self.location.line += 1;
                            self.location.column = 1;
                        }
                        _ => {
                            self.location.column += 1;
                        }
                    }
                    Some(c)
                }
                None => None,
            }
        }

        pub fn location(&self) -> Location {
            self.location.clone()
        }
    }

    fn tokenize_number(chars: &mut CharsWithLocation) -> Result<Token, ParserError> {
        let mut number = String::new();
        let mut has_dot = false;
        let location = chars.location();
        if let Some(&c) = chars.peek() {
            if c == '+' || c == '-' {
                number.push(chars.next().unwrap());
            }
        }
        while let Some(&c) = chars.peek() {
            if c.is_digit(10) {
                number.push(chars.next().unwrap());
            } else if !has_dot && c == '.' {
                has_dot = true;
                number.push(chars.next().unwrap());
            } else {
                break;
            }
        }
        if let Ok(number) = number.parse() {
            Ok(Token::new(TokenValue::Number(number), location))
        } else {
            Err(ParserError::new(
                &format!("Cannot parse '{}' as number", number),
                &location,
            ))
        }
    }

    fn tokenize_string(chars: &mut CharsWithLocation) -> Result<Token, ParserError> {
        let location = chars.location();
        let start_char = chars.next().unwrap();
        let mut string = String::new();
        while let Some(c) = chars.next() {
            if c == start_char {
                return Ok(Token::new(TokenValue::String(string), location));
            }
            string.push(c);
        }
        return Err(ParserError::new("Unterminated string", &location));
    }

    pub fn tokenize(input: &str) -> Result<Vec<Token>, ParserError> {
        let mut tokens = Vec::new();
        let mut chars = CharsWithLocation::new(input);

        while let Some(&c) = chars.peek() {
            match c {
                ' ' | '\t' | '\n' | '\r' => {
                    // Skip whitespace
                }
                '/' => {
                    chars.next();
                    // Consume comments beginning with '//' to end of line
                    match chars.peek() {
                        Some('/') => {
                            chars.next();
                            while let Some(&c) = chars.peek() {
                                if c == '\n' {
                                    break;
                                }
                                chars.next();
                            }
                        }
                        _ => {
                            return Err(ParserError::new(
                                &format!("Expected a second '/' to start a comment"),
                                &chars.location(),
                            ))
                        }
                    };
                }
                '{' => tokens.push(Token::new(TokenValue::LeftBrace, chars.location())),
                '}' => tokens.push(Token::new(TokenValue::RightBrace, chars.location())),
                '[' => tokens.push(Token::new(TokenValue::LeftBracket, chars.location())),
                ']' => tokens.push(Token::new(TokenValue::RightBracket, chars.location())),
                '(' => tokens.push(Token::new(TokenValue::LeftParen, chars.location())),
                ')' => tokens.push(Token::new(TokenValue::RightParen, chars.location())),
                ',' => tokens.push(Token::new(TokenValue::Comma, chars.location())),
                ':' => tokens.push(Token::new(TokenValue::Colon, chars.location())),
                '"' | '\'' => match tokenize_string(&mut chars) {
                    Ok(token) => {
                        tokens.push(token);
                        continue;
                    }
                    Err(e) => return Err(e),
                },
                '0'..='9' | '+' | '-' => match tokenize_number(&mut chars) {
                    Ok(token) => {
                        tokens.push(token);
                        continue;
                    }
                    Err(e) => return Err(e),
                },
                'a'..='z' | 'A'..='Z' | '_' => {
                    let location = chars.location();
                    let mut identifier = String::new();
                    identifier.push(chars.next().unwrap());
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_alphanumeric() || c == '_' {
                            identifier.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::new(TokenValue::Identifier(identifier), location));
                    continue;
                }
                _ => {
                    return Err(ParserError::new(
                        &format!("Unexpected character: '{}'", c),
                        &chars.location(),
                    ))
                }
            }
            chars.next();
        }
        tokens.push(Token::new(TokenValue::Eof, chars.location()));

        Ok(tokens)
    }
}

pub mod parser {
    use log::warn;

    use super::{
        tokenizer::{ParserError, Token},
        Location,
    };
    use crate::{
        color::Color,
        geometry::{point::Point, vector::Vector},
        scene_parser::tokenizer::TokenValue,
    };
    use std::{
        collections::{HashMap, HashSet},
        convert::{TryFrom, TryInto},
        fmt::Debug,
        iter::Peekable,
    };

    /// Checks that the next token is of the same variant as the expected token.
    /// DOES NOT COMPARE THE TOKEN VALUES
    fn expect_token_variant(
        tokens: &mut Peekable<std::slice::Iter<Token>>,
        expected: &TokenValue,
    ) -> Result<Token, ParserError> {
        // We insert an EOF token at the end to guarantee that peek() and next()
        // on tokens will always succeed, so we need to make sure EOF is never
        // "expected".
        assert_ne!(expected, &TokenValue::Eof);

        // We can unwrap safely due to the EOF token
        let token = tokens.next().unwrap();

        if token.value.is_eq_variant(expected) {
            Ok(token.clone())
        } else {
            Err(ParserError::new(
                &format!("Expected {}, got {}", expected, token.value),
                &token.location,
            ))
        }
    }

    fn expect_number(tokens: &mut Peekable<std::slice::Iter<Token>>) -> Result<f64, ParserError> {
        // We can unwrap safely due to the EOF token
        let token = tokens.next().unwrap();
        match &token.value {
            TokenValue::Number(n) => Ok(*n),
            value => Err(ParserError::new(
                &format!("Expected number, got {}", value),
                &token.location,
            )),
        }
    }

    fn expect_identifier(
        tokens: &mut Peekable<std::slice::Iter<Token>>,
    ) -> Result<String, ParserError> {
        // We can unwrap safely due to the EOF token
        let token = tokens.next().unwrap();
        match &token.value {
            TokenValue::Identifier(name) => Ok(name.clone()),
            value => Err(ParserError::new(
                &format!("Expected identifier, got {}", value),
                &token.location,
            )),
        }
    }

    // TODO: Add location to raw values
    #[derive(PartialEq, Debug)]
    pub enum RawValue {
        Number(f64),
        String(String),
        Vector(Vector),
        Point(Point),
        Color(Color),
        Map(RawValueMap),
        TypedMap(TypedRawValueMap),
        Array(RawValueArray),
    }

    impl RawValue {
        /// RawValue := Number | String | Vector | Color | Map | TypedMap | Array
        pub fn from_tokens(
            tokens: &mut Peekable<std::slice::Iter<Token>>,
        ) -> Result<RawValue, ParserError> {
            // We can unwrap safely due to the EOF token
            let token = *tokens.peek().unwrap();
            match &token.value {
                TokenValue::Number(n) => {
                    tokens.next();
                    Ok(RawValue::Number(*n))
                }
                TokenValue::String(s) => {
                    tokens.next();
                    Ok(RawValue::String(s.to_string()))
                }
                TokenValue::Identifier(name) => {
                    tokens.next();

                    let opener_token = *tokens.peek().unwrap();
                    match &opener_token.value {
                        &TokenValue::LeftParen => match name.as_str() {
                            "Vector" => {
                                expect_token_variant(tokens, &TokenValue::LeftParen)?;
                                let x = expect_number(tokens)?;
                                expect_token_variant(tokens, &TokenValue::Comma)?;
                                let y = expect_number(tokens)?;
                                expect_token_variant(tokens, &TokenValue::Comma)?;
                                let z = expect_number(tokens)?;
                                expect_token_variant(tokens, &TokenValue::RightParen)?;
                                Ok(RawValue::Vector(Vector(x, y, z)))
                            }
                            "Point" => {
                                expect_token_variant(tokens, &TokenValue::LeftParen)?;
                                let x = expect_number(tokens)?;
                                expect_token_variant(tokens, &TokenValue::Comma)?;
                                let y = expect_number(tokens)?;
                                expect_token_variant(tokens, &TokenValue::Comma)?;
                                let z = expect_number(tokens)?;
                                expect_token_variant(tokens, &TokenValue::RightParen)?;
                                Ok(RawValue::Point(Point(x, y, z)))
                            }
                            "Color" => {
                                expect_token_variant(tokens, &TokenValue::LeftParen)?;
                                let r = expect_number(tokens)?;
                                expect_token_variant(tokens, &TokenValue::Comma)?;
                                let g = expect_number(tokens)?;
                                expect_token_variant(tokens, &TokenValue::Comma)?;
                                let b = expect_number(tokens)?;
                                expect_token_variant(tokens, &TokenValue::RightParen)?;
                                Ok(RawValue::Color(Color { r, g, b }))
                            }
                            _ => Ok(RawValue::TypedMap(TypedRawValueMap::from_tokens(tokens)?)),
                        },
                        &TokenValue::LeftBrace => Ok(RawValue::TypedMap(TypedRawValueMap {
                            name: name.to_string(),
                            map: RawValueMap::from_tokens(tokens)?,
                            used_keys: HashSet::new(),
                        })),
                        value => Err(ParserError::new(
                            &format!("Expected '(' or '{{', got {}", value),
                            &opener_token.location,
                        )),
                    }
                }
                TokenValue::LeftBrace => Ok(RawValue::Map(RawValueMap::from_tokens(tokens)?)),
                TokenValue::LeftBracket => Ok(RawValue::Array(RawValueArray::from_tokens(tokens)?)),
                value => Err(ParserError::new(
                    &format!("Expected a raw value. Got {}", value),
                    &token.location,
                )),
            }
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct RawValueMap {
        pub location: Location,
        pub map: HashMap<String, RawValue>,
    }

    impl RawValueMap {
        /// Parses a map with raw values
        ///
        /// Map := '{' Entries '}'
        /// Entries := ɸ | Entry (',' Entry)* ','?
        /// Entry := Identifier ':' RawValue
        pub fn from_tokens(
            tokens: &mut Peekable<std::slice::Iter<Token>>,
        ) -> Result<Self, ParserError> {
            let mut map = HashMap::new();

            let start_token = expect_token_variant(tokens, &TokenValue::LeftBrace)?;
            let location = start_token.location;
            loop {
                // Parse an entry
                // We can unwrap safely due to the EOF token
                let token = tokens.peek().unwrap();
                match &token.value {
                    TokenValue::RightParen => break,
                    TokenValue::Identifier(key) => {
                        tokens.next();

                        expect_token_variant(tokens, &TokenValue::Colon)?;

                        let value = RawValue::from_tokens(tokens)?;
                        if map.insert(key.to_string(), value).is_some() {
                            return Err(ParserError::new(
                                &format!("Duplicate key {}", key),
                                &location,
                            ));
                        }
                    }
                    _ => break,
                }

                // Parse a comma
                // We can unwrap safely due to the EOF token
                let token = tokens.peek().unwrap();
                match token.value {
                    TokenValue::RightBrace => break,
                    TokenValue::Comma => {
                        tokens.next();
                    }
                    _ => break,
                }
            }
            expect_token_variant(tokens, &TokenValue::RightBrace)?;

            Ok(RawValueMap { map, location })
        }

        pub fn get<'a, T>(&'a mut self, key: &str) -> Result<T, ParserError>
        where
            T: TryFrom<&'a mut RawValue, Error = ParserError>,
        {
            let location = self.location.clone();
            self.map
                .get_mut(key)
                .ok_or(ParserError::new(
                    &format!("{} not found in map", key),
                    &self.location,
                ))?
                .try_into()
                .map_err(|e: ParserError| {
                    ParserError::new(
                        &format!(
                            "Error converting map value for '{}' to expected type: {}",
                            key, e.message
                        ),
                        &e.location.unwrap_or(location),
                    )
                })
        }

        pub fn get_or<'a, T>(&'a mut self, key: &str, default: T) -> Result<T, ParserError>
        where
            T: TryFrom<&'a mut RawValue, Error = ParserError>,
        {
            let location = self.location.clone();
            match self.map.get_mut(key) {
                Some(v) => v.try_into().map_err(|e: ParserError| {
                    ParserError::new(
                        &format!(
                            "Error converting map value for '{}' to expected type: {}",
                            key, e.message
                        ),
                        &e.location.unwrap_or(location),
                    )
                }),
                None => Ok(default),
            }
        }

        pub fn has(&self, key: &str) -> bool {
            self.map.contains_key(key)
        }
    }

    #[derive(PartialEq, Debug)]
    pub struct TypedRawValueMap {
        pub name: String,
        map: RawValueMap,
        used_keys: HashSet<String>,
    }

    impl TypedRawValueMap {
        /// TypedMap := Identifier Map
        fn from_tokens(
            tokens: &mut Peekable<std::slice::Iter<Token>>,
        ) -> Result<Self, ParserError> {
            let name = expect_identifier(tokens)?;
            let map = RawValueMap::from_tokens(tokens)?;
            Ok(TypedRawValueMap {
                name,
                map,
                used_keys: HashSet::new(),
            })
        }

        pub fn new(name: String, map: RawValueMap) -> Self {
            TypedRawValueMap {
                name,
                map,
                used_keys: HashSet::new(),
            }
        }

        pub fn has(&mut self, key: &str) -> bool {
            self.used_keys.insert(key.to_string());
            self.map.has(key)
        }

        pub fn get<'a, T>(&'a mut self, key: &str) -> Result<T, ParserError>
        where
            T: TryFrom<&'a mut RawValue, Error = ParserError>,
        {
            self.used_keys.insert(key.to_string());
            self.map.get(key)
        }

        pub fn get_or<'a, T>(&'a mut self, key: &str, default: T) -> Result<T, ParserError>
        where
            T: TryFrom<&'a mut RawValue, Error = ParserError>,
        {
            self.used_keys.insert(key.to_string());
            self.map.get_or(key, default)
        }

        pub fn location(&self) -> &Location {
            &self.map.location
        }
    }

    impl Drop for TypedRawValueMap {
        fn drop(&mut self) {
            let unused_keys: Vec<&String> = self
                .map
                .map
                .keys()
                .filter(|key| !self.used_keys.contains(key.as_str()))
                .collect();
            if unused_keys.len() > 0 {
                warn!(
                    "Found unused key(s) {:?} in {} at {}",
                    unused_keys, self.name, self.map.location
                );
            }
        }
    }

    #[derive(PartialEq, Debug)]
    pub struct RawValueArray {
        pub array: Vec<RawValue>,
    }

    impl RawValueArray {
        /// Array := '[' Items ']'
        /// Items := ɸ | RawValue (',' Items)* ','?
        pub fn from_tokens(
            tokens: &mut Peekable<std::slice::Iter<Token>>,
        ) -> Result<Self, ParserError> {
            let mut array = Vec::new();

            expect_token_variant(tokens, &TokenValue::LeftBracket)?;
            loop {
                // Parse an item
                // We can unwrap safely due to the EOF token
                let token = tokens.peek().unwrap();
                match token.value {
                    TokenValue::RightBracket => break,
                    _ => {
                        let value = RawValue::from_tokens(tokens)?;
                        array.push(value);
                    }
                }

                // Parse a comma
                // We can unwrap safely due to the EOF token
                let token = tokens.peek().unwrap();
                match token.value {
                    TokenValue::RightBracket => break,
                    TokenValue::Comma => {
                        tokens.next();
                    }
                    _ => break,
                }
            }
            expect_token_variant(tokens, &TokenValue::RightBracket)?;

            Ok(RawValueArray { array })
        }
    }

    /// Methods for converting RawValues into various concrete values

    impl TryFrom<&mut RawValue> for f64 {
        type Error = ParserError;
        fn try_from(value: &mut RawValue) -> Result<Self, Self::Error> {
            match value {
                RawValue::Number(value) => Ok(*value),
                _ => Err(ParserError::without_location(&format!(
                    "Cannot get Number, found {:?}",
                    value
                ))),
            }
        }
    }

    impl TryFrom<&mut RawValue> for usize {
        type Error = ParserError;
        fn try_from(value: &mut RawValue) -> Result<Self, Self::Error> {
            match value {
                RawValue::Number(value) => Ok(*value as usize),
                _ => Err(ParserError::without_location(&format!(
                    "Cannot get Number, found {:?}",
                    value
                ))),
            }
        }
    }

    impl TryFrom<&mut RawValue> for String {
        type Error = ParserError;
        fn try_from(value: &mut RawValue) -> Result<Self, Self::Error> {
            match value {
                RawValue::String(value) => Ok(value.clone()),
                _ => Err(ParserError::without_location(&format!(
                    "Cannot get String, found {:?}",
                    value
                ))),
            }
        }
    }

    impl TryFrom<&mut RawValue> for Vector {
        type Error = ParserError;
        fn try_from(value: &mut RawValue) -> Result<Self, Self::Error> {
            match value {
                RawValue::Vector(value) => Ok(*value),
                _ => Err(ParserError::without_location(&format!(
                    "Cannot get Vector, found {:?}",
                    value
                ))),
            }
        }
    }

    impl TryFrom<&mut RawValue> for Point {
        type Error = ParserError;
        fn try_from(value: &mut RawValue) -> Result<Self, Self::Error> {
            match value {
                RawValue::Point(value) => Ok(*value),
                _ => Err(ParserError::without_location(&format!(
                    "Cannot get Point, found {:?}",
                    value
                ))),
            }
        }
    }

    impl TryFrom<&mut RawValue> for Color {
        type Error = ParserError;
        fn try_from(value: &mut RawValue) -> Result<Self, Self::Error> {
            match value {
                RawValue::Color(value) => Ok(*value),
                _ => Err(ParserError::without_location(&format!(
                    "Cannot get Color, found {:?}",
                    value
                ))),
            }
        }
    }

    impl<'a: 'b, 'b> TryFrom<&'a mut RawValue> for &'b mut TypedRawValueMap {
        type Error = ParserError;
        fn try_from(value: &'a mut RawValue) -> Result<Self, Self::Error> {
            match value {
                RawValue::TypedMap(value) => Ok(value),
                _ => Err(ParserError::without_location(&format!(
                    "Cannot get TypedRawValueMap, found {:?}",
                    value
                ))),
            }
        }
    }

    /// Converts a RawValueMap into a hashmap with values of given type.
    /// Useful for objects where the keys are names of things of the same type.
    /// For e.g. {materials: {foo: Emissive {...}, bar: Plastic {...}}}
    impl<'a, 'b, T> TryFrom<&'a mut RawValue> for HashMap<String, T>
    where
        'a: 'b,
        T: 'b + TryFrom<&'a mut RawValue, Error = ParserError>,
    {
        type Error = ParserError;
        fn try_from(value: &'a mut RawValue) -> Result<Self, Self::Error> {
            let map = match value {
                RawValue::Map(map) => Ok(map),
                _ => Err(ParserError::without_location(&format!(
                    "Cannot get Map, found {:?}",
                    value
                ))),
            }?;
            let mut result = HashMap::new();
            for (key, value) in map.map.iter_mut() {
                let t: T = value.try_into()?;
                result.insert(key.clone(), t);
            }
            Ok(result)
        }
    }

    /// Converts a RawValueArray into a vec with values of given type.
    impl<'a, 'b, T> TryFrom<&'a mut RawValue> for Vec<T>
    where
        'a: 'b,
        T: 'b + TryFrom<&'a mut RawValue, Error = ParserError>,
    {
        type Error = ParserError;
        fn try_from(value: &'a mut RawValue) -> Result<Self, Self::Error> {
            let array = match value {
                RawValue::Array(array) => Ok(array),
                _ => Err(ParserError::without_location(&format!(
                    "Cannot get Array, found {:?}",
                    value
                ))),
            }?;
            let mut result = Vec::new();
            for value in array.array.iter_mut() {
                let t: T = value.try_into()?;
                result.push(t);
            }
            Ok(result)
        }
    }
}

pub mod scene_parser {
    use super::{
        parser::{RawValue, RawValueMap, TypedRawValueMap},
        tokenizer::{tokenize, ParserError},
        Location,
    };
    use crate::{
        camera::Camera,
        color::Color,
        film::Film,
        geometry::{point::Point, vector::Vector},
        light::Light,
        material::Material,
        obj::load_obj,
        primitive::Primitive,
        scene::Scene,
        shape::Shape,
        texture::{FromPixel, Texture},
    };
    use std::{collections::HashMap, convert::TryFrom, sync::Arc};

    const DEFAULT_MAX_DEPTH: usize = 8;
    const DEFAULT_NUM_SAMPLES: usize = 4;
    const DEFAULT_FOCAL_DISTANCE: f64 = 1e6;

    /// RawValue -> Camera
    impl TryFrom<&mut RawValue> for Camera {
        type Error = ParserError;
        fn try_from(value: &mut RawValue) -> Result<Self, Self::Error> {
            let typed_map = match value {
                RawValue::TypedMap(typed_map) => Ok(typed_map),
                _ => Err(ParserError::without_location(&format!(
                    "Cannot get Camera, found {:?}",
                    value
                ))),
            }?;
            let film: Film = typed_map.get("film")?;
            let origin: Point = typed_map.get("origin")?;
            let target: Point = typed_map.get("target")?;
            let up: Vector = typed_map.get("up")?;
            let lens_radius: f64 = typed_map.get_or("lens_radius", 0.0)?;
            let focal_distance: f64 = typed_map.get_or("focal_distance", DEFAULT_FOCAL_DISTANCE)?;
            match typed_map.name.as_str() {
                "Perspective" => {
                    let fov: f64 = typed_map.get("fov")?;

                    Ok(Camera::perspective(
                        film,
                        origin,
                        target,
                        up,
                        fov,
                        lens_radius,
                        focal_distance,
                    ))
                }
                "Orthographic" => Ok(Camera::orthographic(
                    film,
                    origin,
                    target,
                    up,
                    lens_radius,
                    focal_distance,
                )),
                _ => Err(ParserError::without_location(&format!(
                    "Unknown camera type: {}",
                    typed_map.name
                ))),
            }
        }
    }

    /// RawValue -> Film
    impl TryFrom<&mut RawValue> for Film {
        type Error = ParserError;
        fn try_from(value: &mut RawValue) -> Result<Self, Self::Error> {
            let map = match value {
                RawValue::Map(map) => Ok(map),
                _ => Err(ParserError::without_location(&format!(
                    "Cannot get Film, found {:?}",
                    value
                ))),
            }?;
            let width: usize = map.get("width")?;
            let height: usize = map.get("height")?;

            Ok(Film { width, height })
        }
    }

    /// RawValue -> Light
    impl TryFrom<&mut RawValue> for Arc<Light> {
        type Error = ParserError;
        fn try_from(value: &mut RawValue) -> Result<Self, Self::Error> {
            let typed_map = match value {
                RawValue::TypedMap(typed_map) => Ok(typed_map),
                _ => Err(ParserError::without_location(&format!(
                    "Cannot get Light, found {:?}",
                    value
                ))),
            }?;
            match typed_map.name.as_str() {
                "Point" => {
                    let origin: Point = typed_map.get("origin")?;
                    let intensity: Color = typed_map.get("intensity")?;

                    Ok(Arc::new(Light::Point { origin, intensity }))
                }
                "Distant" => {
                    let direction: Vector = typed_map.get("direction")?;
                    let intensity: Color = typed_map.get("intensity")?;

                    Ok(Arc::new(Light::Distant {
                        direction: direction.normalized(),
                        intensity,
                    }))
                }
                "Infinite" => {
                    let intensity: Color = typed_map.get("intensity")?;

                    Ok(Arc::new(Light::Infinite { intensity }))
                }
                _ => Err(ParserError::without_location(&format!(
                    "Unknown light type: {}",
                    typed_map.name
                ))),
            }
        }
    }

    /// RawValue -> Texture
    ///
    /// Textures support a shorthand where a value of type T will work as a
    /// constant texture instead of writing out a  Texture<T>.
    impl<T> TryFrom<&mut RawValue> for Texture<T>
    where
        T: for<'a> TryFrom<&'a mut RawValue, Error = ParserError> + Copy + FromPixel,
    {
        type Error = ParserError;
        fn try_from(value: &mut RawValue) -> Result<Self, Self::Error> {
            // If a value of type T can be directly constructed from this raw
            // value, create a constant texture using it
            T::try_from(value)
                .map(|value| Texture::Constant(value))
                // Otherwise expect a typed map describing the texture
                .or_else(|_| match value {
                    RawValue::TypedMap(typed_map) => match typed_map.name.as_str() {
                        "Checkerboard" => Ok(Texture::checkerboard(
                            typed_map.get("a")?,
                            typed_map.get("b")?,
                            typed_map.get_or("scale", 1.0)?,
                        )),
                        _ => Err(ParserError::new(
                            &format!("Unknown material type: {}", typed_map.name),
                            &typed_map.location(),
                        )),
                    },
                    _ => Err(ParserError::without_location(&format!(
                        "Cannot get Color, found {:?}",
                        value
                    ))),
                })
        }
    }

    /// RawValue -> Material
    impl TryFrom<&mut RawValue> for Arc<Material> {
        type Error = ParserError;
        fn try_from(value: &mut RawValue) -> Result<Self, Self::Error> {
            let typed_map = match value {
                RawValue::TypedMap(typed_map) => Ok(typed_map),
                _ => Err(ParserError::without_location(&format!(
                    "Cannot get Material, found {:?}",
                    value
                ))),
            }?;
            match typed_map.name.as_str() {
                "Matte" => Ok(Arc::new(Material::new_matte(
                    typed_map.get("reflectance")?,
                    typed_map.get("sigma")?,
                ))),
                "Glass" => Ok(Arc::new(Material::new_glass(
                    typed_map.get("reflectance")?,
                    typed_map.get("transmittance")?,
                    typed_map.get("eta")?,
                ))),
                "Plastic" => Ok(Arc::new(Material::new_plastic(
                    typed_map.get("diffuse")?,
                    typed_map.get("specular")?,
                    typed_map.get("roughness")?,
                ))),
                "Metal" => Ok(Arc::new(Material::new_metal(
                    typed_map.get("eta")?,
                    typed_map.get("k")?,
                ))),
                _ => Err(ParserError::new(
                    &format!("Unknown material type: {}", typed_map.name),
                    &typed_map.location(),
                )),
            }
        }
    }

    /// RawValue -> Shape
    impl TryFrom<&mut RawValue> for Arc<Shape> {
        type Error = ParserError;
        fn try_from(value: &mut RawValue) -> Result<Self, Self::Error> {
            let typed_map = match value {
                RawValue::TypedMap(typed_map) => Ok(typed_map),
                _ => Err(ParserError::without_location(&format!(
                    "Cannot get Shape, found {:?}",
                    value
                ))),
            }?;
            match typed_map.name.as_str() {
                "Sphere" => Ok(Arc::new(Shape::new_sphere(
                    typed_map.get("origin")?,
                    typed_map.get("radius")?,
                ))),
                "Triangle" => Shape::new_triangle(
                    typed_map.get("v0")?,
                    typed_map.get("v1")?,
                    typed_map.get("v2")?,
                )
                .ok_or_else(|| {
                    ParserError::new(
                        &format!("Degenerate triangle: {}", typed_map.name),
                        &typed_map.location(),
                    )
                })
                .map(|shape| Arc::new(shape)),
                "Disk" => Ok(Arc::new(Shape::new_disk(
                    typed_map.get("origin")?,
                    typed_map.get_or("rotate_x", 0.0)?,
                    typed_map.get_or("rotate_y", 0.0)?,
                    typed_map.get("radius")?,
                    typed_map.get_or("inner_radius", 0.0)?,
                ))),
                _ => Err(ParserError::without_location(&format!(
                    "Unknown shape type: {}",
                    typed_map.name
                ))),
            }
        }
    }

    /// TypedRawValueMap -> Primitive
    ///
    /// Unfortunately we can't use the TryFrom pattern for this because it
    /// relies on state (shapes and materials) outside the raw value itself.
    fn create_primitives(
        primitive_def: &mut TypedRawValueMap,
        materials: &HashMap<String, Arc<Material>>,
        shapes: &HashMap<String, Arc<Shape>>,
    ) -> Result<Vec<Arc<Primitive>>, ParserError> {
        match primitive_def.name.as_str() {
            "Shape" => {
                let shape_name: String = primitive_def.get("shape")?;
                let shape = shapes.get(&shape_name).ok_or(ParserError::new(
                    &format!("Cannot find shape named '{}'", shape_name),
                    &primitive_def.location(),
                ))?;

                let primitive = match primitive_def.has("emittance") {
                    false => {
                        let material_name: String = primitive_def.get("material")?;
                        let material = materials.get(&material_name).ok_or(ParserError::new(
                            &format!("Cannot find material named '{}'", material_name),
                            &primitive_def.location(),
                        ))?;
                        Primitive::new(Arc::clone(shape), Arc::clone(material))
                    }
                    true => {
                        let area_light = Arc::new(Light::Area {
                            shape: Arc::clone(shape),
                            emittance: primitive_def.get("emittance")?,
                        });
                        Primitive::new_area_light(Arc::clone(shape), Arc::clone(&area_light))
                    }
                };

                Ok(vec![Arc::new(primitive)])
            }
            "Mesh" => {
                let file_name: String = primitive_def.get("file_name")?;

                let material_name: String = primitive_def.get("fallback_material")?;
                let fallback_material = materials.get(&material_name).ok_or(ParserError::new(
                    &format!("Cannot find material named '{}'", material_name),
                    &primitive_def.location(),
                ))?;

                let primitives = load_obj(&file_name, Arc::clone(fallback_material));

                Ok(primitives)
            }
            _ => Err(ParserError::new(
                &format!("Unknown primitive type: {}", primitive_def.name),
                &primitive_def.location(),
            )),
        }
    }

    pub fn parse_scene(input: &str) -> Result<Scene, ParserError> {
        let tokens = tokenize(input)?;

        let mut tokens = tokens.iter().peekable();
        let mut scene_map = RawValueMap::from_tokens(&mut tokens)?;

        let max_depth: usize = scene_map.get_or("max_depth", DEFAULT_MAX_DEPTH)?;
        let num_samples: usize = scene_map.get_or("num_samples", DEFAULT_NUM_SAMPLES)?;
        let camera: Camera = scene_map.get("camera")?;

        let mut lights: Vec<Arc<Light>> = scene_map.get("lights")?;
        let materials: HashMap<String, Arc<Material>> = scene_map.get("materials")?;
        let shapes: HashMap<String, Arc<Shape>> = scene_map.get("shapes")?;
        let primitive_defs: Vec<&mut TypedRawValueMap> = scene_map.get("primitives")?;

        let mut primitives: Vec<Arc<Primitive>> = Vec::new();
        for primitive_def in primitive_defs {
            for primitive in create_primitives(primitive_def, &materials, &shapes)? {
                if let Some(area_light) = primitive.get_area_light() {
                    lights.push(Arc::clone(area_light));
                }
                primitives.push(primitive);
            }
        }

        if lights.len() == 0 {
            return Err(ParserError::new(
                "No lights in the scene.",
                &Location { line: 0, column: 0 },
            ));
        }

        Ok(Scene::new(
            max_depth,
            num_samples,
            camera,
            lights,
            primitives,
        ))
    }
}
