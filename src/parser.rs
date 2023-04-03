use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    iter::Peekable,
    str::Chars,
};

use crate::{
    camera::{Camera, ProjectionCamera},
    color::Color,
    material::{
        EmissiveMaterial, GlassMaterial, Material, MatteMaterial, MetalMaterial, PlasticMaterial,
    },
    shape::{Shape, Sphere, Triangle},
    vector::Vector,
};

#[derive(Debug, PartialEq, Clone)]
enum Token {
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

impl Token {
    pub fn is_eq_variant(&self, other: &Token) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

#[derive(Debug, PartialEq)]
pub struct ParserError {
    message: String,
}

fn tokenize_number(chars: &mut Peekable<Chars>) -> Result<Token, ParserError> {
    let mut number = String::new();
    let mut has_dot = false;
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
        return Ok(Token::Number(number));
    } else {
        return Err(ParserError {
            message: format!("Cannot parse '{}' as a number", number),
        });
    }
}

fn tokenize_string(chars: &mut Peekable<Chars>) -> Result<Token, ParserError> {
    let start_char = chars.next().unwrap();
    let mut string = String::new();
    while let Some(c) = chars.next() {
        if c == start_char {
            return Ok(Token::String(string));
        }
        string.push(c);
    }
    return Err(ParserError {
        message: "Unterminated string".to_string(),
    });
}

fn tokenize(input: &str) -> Result<Vec<Token>, ParserError> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\n' | '\r' => {
                // Skip whitespace
            }
            '{' => tokens.push(Token::LeftBrace),
            '}' => tokens.push(Token::RightBrace),
            '[' => tokens.push(Token::LeftBracket),
            ']' => tokens.push(Token::RightBracket),
            '(' => tokens.push(Token::LeftParen),
            ')' => tokens.push(Token::RightParen),
            ',' => tokens.push(Token::Comma),
            ':' => tokens.push(Token::Colon),
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
                let mut identifier = String::new();
                identifier.push(chars.next().unwrap());
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        identifier.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Identifier(identifier));
                continue;
            }
            _ => {
                return Err(ParserError {
                    message: format!("Unexpected character: '{}'", c),
                })
            }
        }
        chars.next();
    }
    tokens.push(Token::Eof);

    Ok(tokens)
}

#[cfg(test)]
mod tokenizer_tests {
    use crate::parser::{tokenize, ParserError, Token};

    #[test]
    fn simple() {
        assert_eq!(tokenize("").unwrap(), [Token::Eof]);
        assert_eq!(tokenize(" \r\t\n").unwrap(), [Token::Eof]);
        assert_eq!(
            tokenize("{}").unwrap(),
            [Token::LeftBrace, Token::RightBrace, Token::Eof]
        );
        assert_eq!(
            tokenize("[]").unwrap(),
            [Token::LeftBracket, Token::RightBracket, Token::Eof]
        );
        assert_eq!(
            tokenize("()").unwrap(),
            [Token::LeftParen, Token::RightParen, Token::Eof]
        );
        assert_eq!(tokenize("1").unwrap(), [Token::Number(1.0), Token::Eof]);
        assert_eq!(
            tokenize("'hello'").unwrap(),
            [Token::String("hello".to_string()), Token::Eof]
        );
    }

    #[test]
    fn numbers() {
        assert_eq!(tokenize("1").unwrap(), [Token::Number(1.0), Token::Eof]);
        assert_eq!(tokenize("1.0").unwrap(), [Token::Number(1.0), Token::Eof]);
        assert_eq!(
            tokenize("1.0000").unwrap(),
            [Token::Number(1.0), Token::Eof]
        );
        assert_eq!(
            tokenize("001.0000").unwrap(),
            [Token::Number(1.0), Token::Eof]
        );

        assert_eq!(tokenize("-1").unwrap(), [Token::Number(-1.0), Token::Eof]);
        assert_eq!(tokenize("+1").unwrap(), [Token::Number(1.0), Token::Eof]);
        assert_eq!(tokenize("-1.1").unwrap(), [Token::Number(-1.1), Token::Eof]);
        assert_eq!(tokenize("+1.1").unwrap(), [Token::Number(1.1), Token::Eof]);

        // no expression support
        assert_eq!(
            tokenize("2+3").unwrap(),
            [Token::Number(2.0), Token::Number(3.0), Token::Eof]
        );
        assert_eq!(
            tokenize("4-5").unwrap(),
            [Token::Number(4.0), Token::Number(-5.0), Token::Eof]
        );

        // malformed numbers
        assert_eq!(
            tokenize("9.8.7").expect_err("Expected ParserError"),
            ParserError {
                message: "Unexpected character: '.'".to_string()
            }
        );
    }

    #[test]
    fn strings() {
        [
            // empty
            r#""""#,
            r#"''"#,
            // simple
            "'a'",
            r#""Once upon a midnight dreary""#,
            // mixed quotes
            r#"'"'"#,
            r#""'""#,
        ]
        .iter()
        .for_each(|s| {
            assert_eq!(
                tokenize(s).unwrap(),
                [Token::String(s[1..s.len() - 1].to_string()), Token::Eof],
            );
        });
    }

    #[test]
    fn identifiers() {
        [
            "simple",
            "snake_case",
            "camelCase",
            "SCREAMING_CASE",
            "agent007",
        ]
        .iter()
        .for_each(|s| {
            assert_eq!(
                tokenize(s).unwrap(),
                [Token::Identifier(s.to_string()), Token::Eof]
            );
        });
    }

    #[test]
    fn full() {
        assert_eq!(
            tokenize(
                r#"
{
    max_depth: 8,
    camera: ProjectionCamera {
        origin: Vector(0, 8, -10),
        fov: 5,
    },
    materials: {
        sky: Emissive {
            emittance: Color(0, 10, 60)
        }
    },
    shapes: {
        sky: Sphere {
            center: Vector(0, 0, 0),
            radius: 1000
        }
    },
    primitives: [
        Shape {
            shape: 'sky',
            material: 'sky'
        }
    ]
}
"#
            )
            .unwrap(),
            [
                Token::LeftBrace,
                Token::Identifier("max_depth".to_string()),
                Token::Colon,
                Token::Number(8.0),
                Token::Comma,
                Token::Identifier("camera".to_string()),
                Token::Colon,
                Token::Identifier("ProjectionCamera".to_string()),
                Token::LeftBrace,
                Token::Identifier("origin".to_string()),
                Token::Colon,
                Token::Identifier("Vector".to_string()),
                Token::LeftParen,
                Token::Number(0.0),
                Token::Comma,
                Token::Number(8.0),
                Token::Comma,
                Token::Number(-10.0),
                Token::RightParen,
                Token::Comma,
                Token::Identifier("fov".to_string()),
                Token::Colon,
                Token::Number(5.0),
                Token::Comma,
                Token::RightBrace,
                Token::Comma,
                Token::Identifier("materials".to_string()),
                Token::Colon,
                Token::LeftBrace,
                Token::Identifier("sky".to_string()),
                Token::Colon,
                Token::Identifier("Emissive".to_string()),
                Token::LeftBrace,
                Token::Identifier("emittance".to_string()),
                Token::Colon,
                Token::Identifier("Color".to_string()),
                Token::LeftParen,
                Token::Number(0.0),
                Token::Comma,
                Token::Number(10.0),
                Token::Comma,
                Token::Number(60.0),
                Token::RightParen,
                Token::RightBrace,
                Token::RightBrace,
                Token::Comma,
                Token::Identifier("shapes".to_string()),
                Token::Colon,
                Token::LeftBrace,
                Token::Identifier("sky".to_string()),
                Token::Colon,
                Token::Identifier("Sphere".to_string()),
                Token::LeftBrace,
                Token::Identifier("center".to_string()),
                Token::Colon,
                Token::Identifier("Vector".to_string()),
                Token::LeftParen,
                Token::Number(0.0),
                Token::Comma,
                Token::Number(0.0),
                Token::Comma,
                Token::Number(0.0),
                Token::RightParen,
                Token::Comma,
                Token::Identifier("radius".to_string()),
                Token::Colon,
                Token::Number(1000.0),
                Token::RightBrace,
                Token::RightBrace,
                Token::Comma,
                Token::Identifier("primitives".to_string()),
                Token::Colon,
                Token::LeftBracket,
                Token::Identifier("Shape".to_string()),
                Token::LeftBrace,
                Token::Identifier("shape".to_string()),
                Token::Colon,
                Token::String("sky".to_string()),
                Token::Comma,
                Token::Identifier("material".to_string()),
                Token::Colon,
                Token::String("sky".to_string()),
                Token::RightBrace,
                Token::RightBracket,
                Token::RightBrace,
                Token::Eof
            ]
        )
    }
}

/// Checks that the next token is of the same variant as the expected token.
/// DOES NOT COMPARE THE TOKEN VALUES
fn expect_token_variant(
    tokens: &mut Peekable<std::slice::Iter<Token>>,
    expected: &Token,
) -> Result<Token, ParserError> {
    assert_ne!(expected, &Token::Eof);

    let token = tokens.next().unwrap();

    if token.is_eq_variant(expected) {
        Ok(token.clone())
    } else {
        Err(ParserError {
            message: format!("Expected {:?}, got {:?}", expected, token),
        })
    }
}

fn expect_number(tokens: &mut Peekable<std::slice::Iter<Token>>) -> Result<f64, ParserError> {
    match tokens.next() {
        None => Err(ParserError {
            message: "Unexpected end of input".to_string(),
        }),
        Some(Token::Number(n)) => Ok(*n),
        Some(token) => Err(ParserError {
            message: format!("Expected number, got {:?}", token),
        }),
    }
}

fn expect_identifier(
    tokens: &mut Peekable<std::slice::Iter<Token>>,
) -> Result<String, ParserError> {
    match tokens.next() {
        None => Err(ParserError {
            message: "Unexpected end of input".to_string(),
        }),
        Some(Token::Identifier(name)) => Ok(name.clone()),
        Some(token) => Err(ParserError {
            message: format!("Expected identifier, got {:?}", token),
        }),
    }
}

#[derive(Debug, PartialEq)]
enum RawValue {
    Number(f64),
    String(String),
    Vector(Vector),
    Color(Color),
    Map(RawValueMap),
    TypedMap(TypedRawValueMap),
}

impl TryFrom<&RawValue> for f64 {
    type Error = ParserError;
    fn try_from(value: &RawValue) -> Result<Self, Self::Error> {
        match value {
            RawValue::Number(value) => Ok(*value),
            _ => Err(ParserError {
                message: format!("Cannot get Number, found {:?}", value),
            }),
        }
    }
}

impl TryFrom<&RawValue> for usize {
    type Error = ParserError;
    fn try_from(value: &RawValue) -> Result<Self, Self::Error> {
        match value {
            RawValue::Number(value) => Ok(*value as usize),
            _ => Err(ParserError {
                message: format!("Cannot get Number, found {:?}", value),
            }),
        }
    }
}

impl TryFrom<&RawValue> for String {
    type Error = ParserError;
    fn try_from(value: &RawValue) -> Result<Self, Self::Error> {
        match value {
            RawValue::String(value) => Ok(value.clone()),
            _ => Err(ParserError {
                message: format!("Cannot get String, found {:?}", value),
            }),
        }
    }
}

impl TryFrom<&RawValue> for Vector {
    type Error = ParserError;
    fn try_from(value: &RawValue) -> Result<Self, Self::Error> {
        match value {
            RawValue::Vector(value) => Ok(*value),
            _ => Err(ParserError {
                message: format!("Cannot get Vector, found {:?}", value),
            }),
        }
    }
}

impl TryFrom<&RawValue> for Color {
    type Error = ParserError;
    fn try_from(value: &RawValue) -> Result<Self, Self::Error> {
        match value {
            RawValue::Color(value) => Ok(*value),
            _ => Err(ParserError {
                message: format!("Cannot get Color, found {:?}", value),
            }),
        }
    }
}

impl<'a: 'b, 'b> TryFrom<&'a RawValue> for &'b RawValueMap {
    type Error = ParserError;
    fn try_from(value: &'a RawValue) -> Result<Self, Self::Error> {
        match value {
            RawValue::Map(value) => Ok(value),
            _ => Err(ParserError {
                message: format!("Cannot get Map, found {:?}", value),
            }),
        }
    }
}

impl<'a: 'b, 'b> TryFrom<&'a RawValue> for &'b TypedRawValueMap {
    type Error = ParserError;
    fn try_from(value: &'a RawValue) -> Result<Self, Self::Error> {
        match value {
            RawValue::TypedMap(value) => Ok(value),
            _ => Err(ParserError {
                message: format!("Cannot get TypedMap, found {:?}", value),
            }),
        }
    }
}

impl TryFrom<&RawValue> for Box<dyn Camera> {
    type Error = ParserError;
    fn try_from(value: &RawValue) -> Result<Self, Self::Error> {
        let typed_map = match value {
            RawValue::TypedMap(typed_map) => Ok(typed_map),
            _ => Err(ParserError {
                message: format!("Cannot get Camera, found {:?}", value),
            }),
        }?;
        let map = &typed_map.map;
        match typed_map.name.as_str() {
            "Projection" => {
                let origin: Vector = map.get("origin")?;
                let target: Vector = map.get("target")?;
                let up: Vector = map.get("up")?;
                let focal_distance: f64 = map.get("focal_distance")?;
                let num_samples: usize = map.get("num_samples")?;
                let film_width: usize = map.get("film_width")?;
                let film_height: usize = map.get("film_height")?;

                Ok(Box::new(ProjectionCamera::new(
                    origin,
                    target,
                    up,
                    focal_distance,
                    num_samples,
                    film_width,
                    film_height,
                )))
            }
            _ => Err(ParserError {
                message: format!("Unexpected name for Projection camera: {}", typed_map.name),
            }),
        }
    }
}

impl TryFrom<&RawValue> for Box<dyn Material> {
    type Error = ParserError;
    fn try_from(value: &RawValue) -> Result<Self, Self::Error> {
        let typed_map = match value {
            RawValue::TypedMap(typed_map) => Ok(typed_map),
            _ => Err(ParserError {
                message: format!("Cannot get Material, found {:?}", value),
            }),
        }?;
        let map = &typed_map.map;
        match typed_map.name.as_str() {
            "Emissive" => Ok(Box::new(EmissiveMaterial {
                emittance: map.get("emittance")?,
            })),
            "Matte" => Ok(Box::new(MatteMaterial::new(
                map.get("reflectance")?,
                map.get("sigma")?,
            ))),
            "Glass" => Ok(Box::new(GlassMaterial::new(
                map.get("reflectance")?,
                map.get("transmittance")?,
                map.get("eta")?,
            ))),
            "Plastic" => Ok(Box::new(PlasticMaterial::new(
                map.get("diffuse")?,
                map.get("specular")?,
                map.get("roughness")?,
            ))),
            "Metal" => Ok(Box::new(MetalMaterial::new(map.get("eta")?, map.get("k")?))),
            _ => Err(ParserError {
                message: format!("Unknown material type: {}", typed_map.name),
            }),
        }
    }
}

impl TryFrom<&RawValue> for Box<dyn Shape> {
    type Error = ParserError;
    fn try_from(value: &RawValue) -> Result<Self, Self::Error> {
        let typed_map = match value {
            RawValue::TypedMap(typed_map) => Ok(typed_map),
            _ => Err(ParserError {
                message: format!("Cannot get Shape, found {:?}", value),
            }),
        }?;
        let map = &typed_map.map;
        match typed_map.name.as_str() {
            "Sphere" => Ok(Box::new(Sphere::new(
                map.get("origin")?,
                map.get("radius")?,
            ))),
            "Triangle" => Ok(Box::new(Triangle::new(
                map.get("v0")?,
                map.get("v1")?,
                map.get("v2")?,
            ))),
            _ => Err(ParserError {
                message: format!("Unknown shape type: {}", typed_map.name),
            }),
        }
    }
}

/// Converts a (unnamed) raw map into a hashmap with values of given type.
/// Useful for objects where the keys are names of things of the same type.
/// For e.g. {materials: {foo: Emissive {...}, bar: Plastic {...}}}
impl<'a, 'b, T> TryFrom<&'a RawValue> for HashMap<String, T>
where
    'a: 'b,
    T: 'b + TryFrom<&'a RawValue, Error = ParserError>,
{
    type Error = ParserError;
    fn try_from(value: &'a RawValue) -> Result<Self, Self::Error> {
        let map = match value {
            RawValue::Map(map) => Ok(map),
            _ => Err(ParserError {
                message: format!("Cannot get Map, found {:?}", value),
            }),
        }?;
        let mut result = HashMap::new();
        for (key, value) in map.map.iter() {
            let t: T = value.try_into()?;
            result.insert(key.clone(), t);
        }
        Ok(result)
    }
}

fn parse_raw_value(
    tokens: &mut Peekable<std::slice::Iter<Token>>,
) -> Result<RawValue, ParserError> {
    match tokens.peek() {
        Some(Token::Number(n)) => {
            tokens.next();
            Ok(RawValue::Number(*n))
        }
        Some(Token::String(s)) => {
            tokens.next();
            Ok(RawValue::String(s.to_string()))
        }
        Some(Token::Identifier(name)) => match name.as_str() {
            "Vector" => {
                tokens.next();
                expect_token_variant(tokens, &Token::LeftParen)?;
                let x = expect_number(tokens)?;
                expect_token_variant(tokens, &Token::Comma)?;
                let y = expect_number(tokens)?;
                expect_token_variant(tokens, &Token::Comma)?;
                let z = expect_number(tokens)?;
                expect_token_variant(tokens, &Token::RightParen)?;
                Ok(RawValue::Vector(Vector(x, y, z)))
            }
            "Color" => {
                tokens.next();
                expect_token_variant(tokens, &Token::LeftParen)?;
                let r = expect_number(tokens)?;
                expect_token_variant(tokens, &Token::Comma)?;
                let g = expect_number(tokens)?;
                expect_token_variant(tokens, &Token::Comma)?;
                let b = expect_number(tokens)?;
                expect_token_variant(tokens, &Token::RightParen)?;
                Ok(RawValue::Color(Color { r, g, b }))
            }
            _ => Ok(RawValue::TypedMap(TypedRawValueMap::from_tokens(tokens)?)),
        },
        Some(Token::LeftBrace) => Ok(RawValue::Map(RawValueMap::from_tokens(tokens)?)),
        Some(token) => Err(ParserError {
            message: format!("Expected number, string, or identifier. Got: {:?}", token),
        }),
        None => Err(ParserError {
            message: "Unexpected end of input".to_string(),
        }),
    }
}

#[derive(Debug, PartialEq)]
struct RawValueMap {
    map: HashMap<String, RawValue>,
}

impl RawValueMap {
    /// Parses a map with raw values
    ///
    /// RawMap := '{' RawMapEntries '}'
    /// RawMapEntries := ɸ | RawMapEntry (',' RawMapEntry)*
    /// RawMapEntry := Identifier ':' RawValue
    fn from_tokens(tokens: &mut Peekable<std::slice::Iter<Token>>) -> Result<Self, ParserError> {
        expect_token_variant(tokens, &Token::LeftBrace)?;

        let mut map = HashMap::new();
        let mut needs_comma = true;

        loop {
            match tokens.peek() {
                Some(Token::RightParen) => break,
                Some(Token::Comma) => {
                    if !needs_comma {
                        break;
                    }
                    tokens.next();
                }
                _ => {}
            }

            match tokens.peek() {
                Some(Token::Identifier(key)) => {
                    tokens.next();

                    expect_token_variant(tokens, &Token::Colon)?;

                    let value = parse_raw_value(tokens)?;
                    if map.insert(key.to_string(), value).is_some() {
                        return Err(ParserError {
                            message: format!("Duplicate key {}", key),
                        });
                    }

                    needs_comma = true;
                }
                _ => break,
            }
        }

        expect_token_variant(tokens, &Token::RightBrace)?;

        Ok(RawValueMap { map })
    }

    fn get<'a, T>(&'a self, key: &str) -> Result<T, ParserError>
    where
        T: TryFrom<&'a RawValue, Error = ParserError>,
    {
        self.map
            .get(key)
            .ok_or(ParserError {
                message: format!("{} not found in map", key),
            })?
            .try_into()
    }
}

#[derive(Debug, PartialEq)]
struct TypedRawValueMap {
    name: String,
    map: RawValueMap,
}

impl TypedRawValueMap {
    fn from_tokens(tokens: &mut Peekable<std::slice::Iter<Token>>) -> Result<Self, ParserError> {
        let name = expect_identifier(tokens)?;
        let map = RawValueMap::from_tokens(tokens)?;
        Ok(TypedRawValueMap { name, map })
    }
}

fn parse_scene(input: &str) -> Result<(), ParserError> {
    let tokens = tokenize(input)?;

    let mut tokens = tokens.iter().peekable();
    let scene_map = RawValueMap::from_tokens(&mut tokens)?;
    println!("{:?}", scene_map);

    let max_depth: usize = scene_map.get("max_depth")?;
    let camera: Box<dyn Camera> = scene_map.get("camera")?;
    let materials: HashMap<String, Box<dyn Material>> = scene_map.get("materials")?;
    let shapes: HashMap<String, Box<dyn Shape>> = scene_map.get("shapes")?;

    println!("{} {:?} {:?}", max_depth, camera, materials);

    Ok(())
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn raw_value() {
        [
            ("1.23", RawValue::Number(1.23)),
            ("'hello'", RawValue::String("hello".to_string())),
            (
                "Vector(1, -2, 3.1)",
                RawValue::Vector(Vector(1.0, -2.0, 3.1)),
            ),
            (
                "Color(0, 0.5, 1)",
                RawValue::Color(Color {
                    r: 0.0,
                    g: 0.5,
                    b: 1.0,
                }),
            ),
            (
                "{}",
                RawValue::Map(RawValueMap {
                    map: HashMap::new(),
                }),
            ),
            (
                "{ x: 1, y: 'z' }",
                RawValue::Map(RawValueMap {
                    map: HashMap::from([
                        ("x".to_string(), RawValue::Number(1.0)),
                        ("y".to_string(), RawValue::String("z".to_string())),
                    ]),
                }),
            ),
            (
                "Sphere { center: Vector(0, 0, 0), radius: 1000 }",
                RawValue::TypedMap(TypedRawValueMap {
                    name: "Sphere".to_string(),
                    map: RawValueMap {
                        map: HashMap::from([
                            ("center".to_string(), RawValue::Vector(Vector::O)),
                            ("radius".to_string(), RawValue::Number(1000.0)),
                        ]),
                    },
                }),
            ),
        ]
        .iter()
        .for_each(|(s, v)| {
            match parse_raw_value(&mut tokenize(s).unwrap().iter().peekable()) {
                Ok(result) => assert_eq!(result, *v, "Failed to parse {}", s),
                Err(e) => panic!("Failed to parse {}: {:?}", s, e),
            }
        });

        assert_eq!(
            parse_raw_value(&mut tokenize("x").unwrap().iter().peekable()),
            Err(ParserError {
                message: "Expected LeftBrace, got Eof".to_string()
            })
        );
    }

    #[test]
    fn raw_map() {
        [
            (
                "{}",
                RawValueMap {
                    map: HashMap::new(),
                },
            ),
            (
                "{ hello: 'world' }",
                RawValueMap {
                    map: HashMap::from([(
                        "hello".to_string(),
                        RawValue::String("world".to_string()),
                    )]),
                },
            ),
            (
                "{ x: 1, y: 'z', v: Vector(1,2,3), c: Color(1,0,0) }",
                RawValueMap {
                    map: HashMap::from([
                        ("x".to_string(), RawValue::Number(1.0)),
                        ("y".to_string(), RawValue::String("z".to_string())),
                        ("v".to_string(), RawValue::Vector(Vector(1.0, 2.0, 3.0))),
                        (
                            "c".to_string(),
                            RawValue::Color(Color {
                                r: 1.0,
                                g: 0.0,
                                b: 0.0,
                            }),
                        ),
                    ]),
                },
            ),
        ]
        .iter()
        .for_each(|(input, entries)| {
            assert_eq!(
                RawValueMap::from_tokens(&mut tokenize(input).unwrap().iter().peekable()).unwrap(),
                *entries
            )
        });

        assert_eq!(
            RawValueMap::from_tokens(&mut tokenize("{ x: 1, x: 2 }").unwrap().iter().peekable()),
            Err(ParserError {
                message: "Duplicate key x".to_string()
            })
        );
    }

    #[test]
    fn scene() {
        parse_scene(
            "
{
    max_depth: 5,
    camera: Projection {
        origin: Vector(0, 8, -10),
        target: Vector(1, 1.25, 12),
        up: Vector(0, 1, 0),
        focal_distance: 5,
        film_width: 896,
        film_height: 560,
        num_samples: 10
    },
    materials: {
        sky: Emissive {
            emittance: Color(0, 10, 60)
        },
        ground: Matte {
            reflectance: Color(1, 1, 1),
            sigma: 0
        },
        glass: Glass {
            reflectance: Color(1, 1, 1),
            transmittance: Color(1, 1, 1),
            eta: 1.75
        },
        light: Emissive {
            emittance: Color(255, 230, 20)
        }
    },
    shapes: {
        sky: Sphere {
            origin: Vector(0, 0, 0),
            radius: 1000
        },
        ground: Sphere {
            origin: Vector(0, -10000, 10),
            radius: 10000
        },
        glass: Sphere {
            origin: Vector(0, 1.5, 12.5),
            radius: 1.5
        },
        light: Sphere {
            origin: Vector(-3, 4, 13.5),
            radius: 0.5
        }
    }
}
",
        )
        .unwrap();
    }
}
