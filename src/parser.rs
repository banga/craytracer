use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    iter::Peekable,
    str::Chars,
    sync::Arc,
};

use crate::{
    camera::Camera, color::Color, material::Material, primitive::Primitive, scene::Scene,
    shape::Shape, vector::Vector,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
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
    pub message: String,
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

pub fn tokenize(input: &str) -> Result<Vec<Token>, ParserError> {
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
pub enum RawValue {
    Number(f64),
    String(String),
    Vector(Vector),
    Color(Color),
    Map(RawValueMap),
    TypedMap(TypedRawValueMap),
    Array(RawValueArray),
}

/// RawValue := Number | String | Vector | Color | Map | TypedMap | Array
pub fn parse_raw_value(
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
        Some(Token::LeftBracket) => Ok(RawValue::Array(RawValueArray::from_tokens(tokens)?)),
        Some(token) => Err(ParserError {
            message: format!(
                "Expected number, string, identifier, map or array. Got: {:?}",
                token
            ),
        }),
        None => Err(ParserError {
            message: "Unexpected end of input".to_string(),
        }),
    }
}

#[derive(Debug, PartialEq)]
pub struct RawValueMap {
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
        let mut needs_comma = true;

        expect_token_variant(tokens, &Token::LeftBrace)?;
        loop {
            match tokens.peek() {
                Some(Token::RightParen) => break,
                Some(Token::Comma) => {
                    if !needs_comma {
                        break;
                    }
                    tokens.next();
                }
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
pub struct TypedRawValueMap {
    pub name: String,
    pub map: RawValueMap,
}

impl TypedRawValueMap {
    /// TypedMap := Identifier Map
    fn from_tokens(tokens: &mut Peekable<std::slice::Iter<Token>>) -> Result<Self, ParserError> {
        let name = expect_identifier(tokens)?;
        let map = RawValueMap::from_tokens(tokens)?;
        Ok(TypedRawValueMap { name, map })
    }
}

#[derive(Debug, PartialEq)]
pub struct RawValueArray {
    pub array: Vec<RawValue>,
}

impl RawValueArray {
    /// Array := '[' Items ']'
    /// Items := ɸ | RawValue (',' Items)* ','?
    fn from_tokens(tokens: &mut Peekable<std::slice::Iter<Token>>) -> Result<Self, ParserError> {
        let mut array = Vec::new();
        let mut needs_comma = true;

        expect_token_variant(tokens, &Token::LeftBracket)?;
        loop {
            match tokens.peek() {
                Some(Token::RightBracket) => break,
                Some(Token::Comma) => {
                    if !needs_comma {
                        break;
                    }
                    tokens.next();
                }
                _ => {
                    let value = parse_raw_value(tokens)?;
                    array.push(value);
                    needs_comma = true;
                }
            }
        }
        expect_token_variant(tokens, &Token::RightBracket)?;

        Ok(RawValueArray { array })
    }
}

/// Methods for converting RawValues into various concrete values

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

/// Camera
impl TryFrom<&RawValue> for Box<Camera> {
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

                Ok(Box::new(Camera::new_projection_camera(
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

/// Material
impl TryFrom<&RawValue> for Arc<Material> {
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
            "Emissive" => Ok(Arc::new(Material::new_emissive(map.get("emittance")?))),
            "Matte" => Ok(Arc::new(Material::new_matte(
                map.get("reflectance")?,
                map.get("sigma")?,
            ))),
            "Glass" => Ok(Arc::new(Material::new_glass(
                map.get("reflectance")?,
                map.get("transmittance")?,
                map.get("eta")?,
            ))),
            "Plastic" => Ok(Arc::new(Material::new_plastic(
                map.get("diffuse")?,
                map.get("specular")?,
                map.get("roughness")?,
            ))),
            "Metal" => Ok(Arc::new(Material::new_metal(
                map.get("eta")?,
                map.get("k")?,
            ))),
            _ => Err(ParserError {
                message: format!("Unknown material type: {}", typed_map.name),
            }),
        }
    }
}

/// Shape
impl TryFrom<&RawValue> for Arc<Shape> {
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
            "Sphere" => Ok(Arc::new(Shape::new_sphere(
                map.get("origin")?,
                map.get("radius")?,
            ))),
            "Triangle" => Ok(Arc::new(Shape::new_triangle(
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

/// Converts a RawValueMap into a hashmap with values of given type.
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

/// Converts a RawValueArray into a vec with values of given type.
impl<'a, 'b, T> TryFrom<&'a RawValue> for Vec<T>
where
    'a: 'b,
    T: 'b + TryFrom<&'a RawValue, Error = ParserError>,
{
    type Error = ParserError;
    fn try_from(value: &'a RawValue) -> Result<Self, Self::Error> {
        let array = match value {
            RawValue::Array(array) => Ok(array),
            _ => Err(ParserError {
                message: format!("Cannot get Array, found {:?}", value),
            }),
        }?;
        let mut result = Vec::new();
        for value in array.array.iter() {
            let t: T = value.try_into()?;
            result.push(t);
        }
        Ok(result)
    }
}

pub fn parse_scene(input: &str) -> Result<Scene, ParserError> {
    let tokens = tokenize(input)?;

    let mut tokens = tokens.iter().peekable();
    let scene_map = RawValueMap::from_tokens(&mut tokens)?;

    let max_depth: usize = scene_map.get("max_depth")?;
    let camera: Box<Camera> = scene_map.get("camera")?;
    let materials: HashMap<String, Arc<Material>> = scene_map.get("materials")?;
    let shapes: HashMap<String, Arc<Shape>> = scene_map.get("shapes")?;
    let primitive_defs: Vec<HashMap<String, String>> = scene_map.get("primitives")?;

    let mut primitives: Vec<Arc<Primitive>> = Vec::new();
    for primitive_def in primitive_defs {
        let shape_name = &primitive_def["shape"];
        let shape = shapes.get(shape_name).ok_or(ParserError {
            message: format!("Cannot find shape named '{}'", shape_name),
        })?;

        let material_name = &primitive_def["shape"];
        let material = materials.get(material_name).ok_or(ParserError {
            message: format!("Cannot find material named '{}'", material_name),
        })?;

        let primitive = Arc::new(Primitive::new_shape_primitive(
            Arc::clone(shape),
            Arc::clone(material),
        ));
        primitives.push(primitive);
    }

    // TODO: move film dimensions to Scene
    let (film_width, film_height) = match *camera {
        Camera::Projection {
            film_width,
            film_height,
            ..
        } => (film_width, film_height),
    };

    Ok(Scene::new(
        max_depth,
        film_width,
        film_height,
        camera,
        primitives,
    ))
}
