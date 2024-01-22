#[cfg(test)]
mod tokenizer {
    use craytracer::scene_parser::{
        tokenizer::{tokenize, ParserError, Token, TokenValue},
        Location,
    };
    use pretty_assertions::assert_eq;

    fn assert_token_values(input: &str, expected: &[TokenValue]) {
        assert_eq!(
            tokenize(input)
                .unwrap()
                .iter()
                .map(|token| token.value.clone())
                .collect::<Vec<TokenValue>>(),
            expected
        );
    }

    fn assert_tokens(input: &str, expected: &[Token]) {
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    fn assert_tokenize_error(input: &str, message: &str, location: &Location) {
        assert_eq!(
            tokenize(input).expect_err("Expected ParserError"),
            ParserError::new(message, location)
        );
    }

    #[test]
    fn simple() {
        assert_token_values("", &[TokenValue::Eof]);
        assert_token_values(" \r\t\n", &[TokenValue::Eof]);
        assert_token_values("// foo", &[TokenValue::Eof]);
        assert_token_values(
            "{}",
            &[
                TokenValue::LeftBrace,
                TokenValue::RightBrace,
                TokenValue::Eof,
            ],
        );
        assert_token_values(
            "[]",
            &[
                TokenValue::LeftBracket,
                TokenValue::RightBracket,
                TokenValue::Eof,
            ],
        );
        assert_token_values(
            "()",
            &[
                TokenValue::LeftParen,
                TokenValue::RightParen,
                TokenValue::Eof,
            ],
        );
        assert_token_values("1", &[TokenValue::Number(1.0), TokenValue::Eof]);
        assert_token_values(
            "'hello'",
            &[TokenValue::String("hello".to_string()), TokenValue::Eof],
        );
    }

    #[test]
    fn comments() {
        assert_token_values("// foo = 'hello'", &[TokenValue::Eof]);
        assert_token_values("//\n", &[TokenValue::Eof]);
        assert_token_values("//\n1", &[TokenValue::Number(1.0), TokenValue::Eof]);
        assert_token_values("1 // one", &[TokenValue::Number(1.0), TokenValue::Eof]);

        assert_tokenize_error(
            "/",
            "Expected a second '/' to start a comment",
            &Location { line: 1, column: 2 },
        );
        assert_tokenize_error(
            "/ /",
            "Expected a second '/' to start a comment",
            &Location { line: 1, column: 2 },
        );
    }

    #[test]
    fn numbers() {
        assert_token_values("1", &[TokenValue::Number(1.0), TokenValue::Eof]);
        assert_token_values("1.0", &[TokenValue::Number(1.0), TokenValue::Eof]);
        assert_token_values("1.0000", &[TokenValue::Number(1.0), TokenValue::Eof]);
        assert_token_values("001.0000", &[TokenValue::Number(1.0), TokenValue::Eof]);

        assert_token_values("-1", &[TokenValue::Number(-1.0), TokenValue::Eof]);
        assert_token_values("+1", &[TokenValue::Number(1.0), TokenValue::Eof]);
        assert_token_values("-1.1", &[TokenValue::Number(-1.1), TokenValue::Eof]);
        assert_token_values("+1.1", &[TokenValue::Number(1.1), TokenValue::Eof]);

        // no expression support
        assert_token_values(
            "2+3",
            &[
                TokenValue::Number(2.0),
                TokenValue::Number(3.0),
                TokenValue::Eof,
            ],
        );
        assert_token_values(
            "4-5",
            &[
                TokenValue::Number(4.0),
                TokenValue::Number(-5.0),
                TokenValue::Eof,
            ],
        );

        // malformed numbers
        assert_tokenize_error(
            "9.8.7",
            "Unexpected character: '.'",
            &Location { line: 1, column: 4 },
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
            assert_token_values(
                s,
                &[
                    TokenValue::String(s[1..s.len() - 1].to_string()),
                    TokenValue::Eof,
                ],
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
            assert_token_values(s, &[TokenValue::Identifier(s.to_string()), TokenValue::Eof]);
        });
    }

    #[test]
    fn locations() {
        assert_tokens(
            "{}",
            &[
                Token::new(TokenValue::LeftBrace, Location { line: 1, column: 1 }),
                Token::new(TokenValue::RightBrace, Location { line: 1, column: 2 }),
                Token::new(TokenValue::Eof, Location { line: 1, column: 3 }),
            ],
        );

        assert_tokens(
            "
{
    x: 1,
    y: ['foo', 3.14],
}",
            &[
                Token::new(TokenValue::LeftBrace, Location { line: 2, column: 1 }),
                Token::new(
                    TokenValue::Identifier("x".to_string()),
                    Location { line: 3, column: 5 },
                ),
                Token::new(TokenValue::Colon, Location { line: 3, column: 6 }),
                Token::new(TokenValue::Number(1.0), Location { line: 3, column: 8 }),
                Token::new(TokenValue::Comma, Location { line: 3, column: 9 }),
                Token::new(
                    TokenValue::Identifier("y".to_string()),
                    Location { line: 4, column: 5 },
                ),
                Token::new(TokenValue::Colon, Location { line: 4, column: 6 }),
                Token::new(TokenValue::LeftBracket, Location { line: 4, column: 8 }),
                Token::new(
                    TokenValue::String("foo".to_string()),
                    Location { line: 4, column: 9 },
                ),
                Token::new(
                    TokenValue::Comma,
                    Location {
                        line: 4,
                        column: 14,
                    },
                ),
                Token::new(
                    TokenValue::Number(3.14),
                    Location {
                        line: 4,
                        column: 16,
                    },
                ),
                Token::new(
                    TokenValue::RightBracket,
                    Location {
                        line: 4,
                        column: 20,
                    },
                ),
                Token::new(
                    TokenValue::Comma,
                    Location {
                        line: 4,
                        column: 21,
                    },
                ),
                Token::new(TokenValue::RightBrace, Location { line: 5, column: 1 }),
                Token::new(TokenValue::Eof, Location { line: 5, column: 2 }),
            ],
        );
    }

    #[test]
    fn full() {
        assert_token_values(
            r#"
{
    camera: ProjectionCamera {
        origin: Point(0, 8, -10),
        up: Vector(0, 1, 0),
        fov: 5,
    },
    materials: {
        sky: Emissive {
            emittance: Color(0, 10, 60)
        }
    },
    shapes: {
        sky: Sphere {
            center: Point(0, 0, 0),
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
"#,
            &[
                TokenValue::LeftBrace,
                TokenValue::Identifier("camera".to_string()),
                TokenValue::Colon,
                TokenValue::Identifier("ProjectionCamera".to_string()),
                TokenValue::LeftBrace,
                TokenValue::Identifier("origin".to_string()),
                TokenValue::Colon,
                TokenValue::Identifier("Point".to_string()),
                TokenValue::LeftParen,
                TokenValue::Number(0.0),
                TokenValue::Comma,
                TokenValue::Number(8.0),
                TokenValue::Comma,
                TokenValue::Number(-10.0),
                TokenValue::RightParen,
                TokenValue::Comma,
                TokenValue::Identifier("up".to_string()),
                TokenValue::Colon,
                TokenValue::Identifier("Vector".to_string()),
                TokenValue::LeftParen,
                TokenValue::Number(0.0),
                TokenValue::Comma,
                TokenValue::Number(1.0),
                TokenValue::Comma,
                TokenValue::Number(0.0),
                TokenValue::RightParen,
                TokenValue::Comma,
                TokenValue::Identifier("fov".to_string()),
                TokenValue::Colon,
                TokenValue::Number(5.0),
                TokenValue::Comma,
                TokenValue::RightBrace,
                TokenValue::Comma,
                TokenValue::Identifier("materials".to_string()),
                TokenValue::Colon,
                TokenValue::LeftBrace,
                TokenValue::Identifier("sky".to_string()),
                TokenValue::Colon,
                TokenValue::Identifier("Emissive".to_string()),
                TokenValue::LeftBrace,
                TokenValue::Identifier("emittance".to_string()),
                TokenValue::Colon,
                TokenValue::Identifier("Color".to_string()),
                TokenValue::LeftParen,
                TokenValue::Number(0.0),
                TokenValue::Comma,
                TokenValue::Number(10.0),
                TokenValue::Comma,
                TokenValue::Number(60.0),
                TokenValue::RightParen,
                TokenValue::RightBrace,
                TokenValue::RightBrace,
                TokenValue::Comma,
                TokenValue::Identifier("shapes".to_string()),
                TokenValue::Colon,
                TokenValue::LeftBrace,
                TokenValue::Identifier("sky".to_string()),
                TokenValue::Colon,
                TokenValue::Identifier("Sphere".to_string()),
                TokenValue::LeftBrace,
                TokenValue::Identifier("center".to_string()),
                TokenValue::Colon,
                TokenValue::Identifier("Point".to_string()),
                TokenValue::LeftParen,
                TokenValue::Number(0.0),
                TokenValue::Comma,
                TokenValue::Number(0.0),
                TokenValue::Comma,
                TokenValue::Number(0.0),
                TokenValue::RightParen,
                TokenValue::Comma,
                TokenValue::Identifier("radius".to_string()),
                TokenValue::Colon,
                TokenValue::Number(1000.0),
                TokenValue::RightBrace,
                TokenValue::RightBrace,
                TokenValue::Comma,
                TokenValue::Identifier("primitives".to_string()),
                TokenValue::Colon,
                TokenValue::LeftBracket,
                TokenValue::Identifier("Shape".to_string()),
                TokenValue::LeftBrace,
                TokenValue::Identifier("shape".to_string()),
                TokenValue::Colon,
                TokenValue::String("sky".to_string()),
                TokenValue::Comma,
                TokenValue::Identifier("material".to_string()),
                TokenValue::Colon,
                TokenValue::String("sky".to_string()),
                TokenValue::RightBrace,
                TokenValue::RightBracket,
                TokenValue::RightBrace,
                TokenValue::Eof,
            ],
        )
    }
}

#[cfg(test)]
mod parser {
    use pretty_assertions::assert_eq;
    use std::{collections::HashMap, sync::Arc};

    use craytracer::{
        camera::Camera,
        color::Color,
        film::Film,
        geometry::{point::Point, vector::Vector, O, Y},
        light::Light,
        material::Material,
        p,
        primitive::Primitive,
        scene::Scene,
        scene_parser::scene_parser::parse_scene,
        scene_parser::tokenizer::tokenize,
        scene_parser::{
            parser::{RawValue, RawValueArray, RawValueMap, TypedRawValueMap},
            Location,
        },
        shape::Shape,
        v,
    };

    fn expect_raw_value(s: &str, expected: RawValue) {
        let result = RawValue::from_tokens(&mut tokenize(s).unwrap().iter().peekable())
            .expect(&format!("{:?} failed to parse", s));
        assert_eq!(result, expected, "{:?} parsed to unexpected value", s);
    }

    fn expect_raw_value_map(s: &str, expected: HashMap<String, RawValue>) {
        expect_raw_value(
            s,
            RawValue::Map(RawValueMap {
                map: expected,
                location: Location { line: 1, column: 1 },
            }),
        );
    }

    fn expect_raw_value_array(s: &str, expected: Vec<RawValue>) {
        expect_raw_value(s, RawValue::Array(RawValueArray { array: expected }));
    }

    fn expect_parse_error(s: &str, error: &str, location: Option<Location>) {
        let result = RawValue::from_tokens(&mut tokenize(s).unwrap().iter().peekable())
            .expect_err(&format!("{:?} parsed succesfully", s));
        assert_eq!(result.message, error);
        assert_eq!(result.location, location);
    }

    #[test]
    fn raw_value() {
        expect_raw_value("1.23", RawValue::Number(1.23));
        expect_raw_value("'hello'", RawValue::String("hello".to_string()));
        expect_raw_value("Vector(1, -2, 3.1)", RawValue::Vector(v!(1, -2, 3.1)));
        expect_raw_value(
            "Color(0, 0.5, 1)",
            RawValue::Color(Color {
                r: 0.0,
                g: 0.5,
                b: 1.0,
            }),
        );
        expect_raw_value(
            "{}",
            RawValue::Map(RawValueMap {
                map: HashMap::new(),
                location: Location { line: 1, column: 1 },
            }),
        );
        expect_raw_value(
            "{ x: 1, y: 'z' }",
            RawValue::Map(RawValueMap {
                map: HashMap::from([
                    ("x".to_string(), RawValue::Number(1.0)),
                    ("y".to_string(), RawValue::String("z".to_string())),
                ]),
                location: Location { line: 1, column: 1 },
            }),
        );
        expect_raw_value(
            "Sphere { center: Point(0, 0, 0), radius: 1000 }",
            RawValue::TypedMap(TypedRawValueMap {
                name: "Sphere".to_string(),
                map: RawValueMap {
                    map: HashMap::from([
                        ("center".to_string(), RawValue::Point(O)),
                        ("radius".to_string(), RawValue::Number(1000.0)),
                    ]),
                    location: Location { line: 1, column: 8 },
                },
            }),
        );
        expect_raw_value("[]", RawValue::Array(RawValueArray { array: Vec::new() }));
        expect_raw_value(
            "[1, 'foo', {}]",
            RawValue::Array(RawValueArray {
                array: vec![
                    RawValue::Number(1.0),
                    RawValue::String("foo".to_string()),
                    RawValue::Map(RawValueMap {
                        map: HashMap::new(),
                        location: Location {
                            line: 1,
                            column: 12,
                        },
                    }),
                ],
            }),
        );

        expect_parse_error(
            "x",
            "Expected '(' or '{', got EOF",
            Some(Location { line: 1, column: 2 }),
        );
        expect_parse_error(
            ",",
            "Expected a raw value. Got ','",
            Some(Location { line: 1, column: 1 }),
        );
    }

    #[test]
    fn raw_value_map() {
        expect_raw_value_map("{}", HashMap::new());
        expect_raw_value_map(
            "{ hello: 'world' }",
            HashMap::from([("hello".to_string(), RawValue::String("world".to_string()))]),
        );
        expect_raw_value_map(
            // Trailing comma
            "{ hello: 'world', }",
            HashMap::from([("hello".to_string(), RawValue::String("world".to_string()))]),
        );
        expect_raw_value_map(
            "{ x: 1, y: 'z', v: Vector(1,2,3), c: Color(1,0,0) }",
            HashMap::from([
                ("x".to_string(), RawValue::Number(1.0)),
                ("y".to_string(), RawValue::String("z".to_string())),
                ("v".to_string(), RawValue::Vector(v!(1, 2, 3))),
                (
                    "c".to_string(),
                    RawValue::Color(Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                    }),
                ),
            ]),
        );

        expect_parse_error(
            "{ x: 1, x: 2 }",
            "Duplicate key x",
            Some(Location { line: 1, column: 1 }),
        );
        expect_parse_error(
            "{ x: 1",
            "Expected '}', got EOF",
            Some(Location { line: 1, column: 7 }),
        );
        expect_parse_error(
            "{ x 1 }",
            "Expected ':', got '1'",
            Some(Location { line: 1, column: 5 }),
        );
        expect_parse_error(
            "{ 1: x }",
            "Expected '}', got '1'",
            Some(Location { line: 1, column: 3 }),
        );
        expect_parse_error(
            "{ x: 1 y: 2 }",
            "Expected '}', got 'y'",
            Some(Location { line: 1, column: 8 }),
        );
    }

    #[test]
    fn raw_value_array() {
        expect_raw_value_array("[]", Vec::new());
        expect_raw_value_array(
            "[1, 2, 3]",
            vec![
                RawValue::Number(1.0),
                RawValue::Number(2.0),
                RawValue::Number(3.0),
            ],
        );
        expect_raw_value_array(
            "[1, 2, 3,]",
            vec![
                RawValue::Number(1.0),
                RawValue::Number(2.0),
                RawValue::Number(3.0),
            ],
        );
        expect_raw_value_array(
            "[1, 'foo', {}]",
            vec![
                RawValue::Number(1.0),
                RawValue::String("foo".to_string()),
                RawValue::Map(RawValueMap {
                    map: HashMap::new(),
                    location: Location {
                        line: 1,
                        column: 12,
                    },
                }),
            ],
        );

        expect_parse_error(
            "[",
            "Expected a raw value. Got EOF",
            Some(Location { line: 1, column: 2 }),
        );
        expect_parse_error(
            "[,]",
            "Expected a raw value. Got ','",
            Some(Location { line: 1, column: 2 }),
        );
        expect_parse_error(
            "[1 2]",
            "Expected ']', got '2'",
            Some(Location { line: 1, column: 4 }),
        );
    }

    #[test]
    fn scene() {
        assert_eq!(
            parse_scene(
                "
{
    max_depth: 3,
    num_samples: 1,
    camera: Perspective {
        origin: Point(0, 0, 0),
        target: Point(0, 0, 1),
        up: Vector(0, 1, 0),
        fov: 60,
        lens_radius: 1,
        focal_distance: 100,
        film: {
            width: 400,
            height: 300
        },    
    },
    lights: [
        Point {
            origin: Point(0, 0, 0),
            intensity: Color(1, 1, 1)
        }
    ],
    materials: {
        ball: Matte {
            reflectance: Color(1, 1, 1),
            sigma: 0
        },
    },
    shapes: {
        ball: Sphere {
            origin: Point(0, 0, 2),
            radius: 1
        },
    },
    primitives: [
       Shape { shape: 'ball', material: 'ball' },
       Mesh { file_name: 'objs/triangle.obj', fallback_material: 'ball' },
    ]
}
",
            )
            .unwrap(),
            Scene::new(
                3,
                1,
                Camera::perspective(
                    Film {
                        width: 400,
                        height: 300
                    },
                    O,
                    Point::new(0, 0, 1),
                    Y,
                    60.0,
                    1.0,
                    100.0
                ),
                vec![Arc::new(Light::Point {
                    origin: O,
                    intensity: Color::WHITE
                }),],
                vec![
                    Arc::new(Primitive::new(
                        Arc::new(Shape::new_sphere(p!(0, 0, 2), 1.0)),
                        Arc::new(Material::new_matte(Color::WHITE, 0.0)),
                    )),
                    Arc::new(Primitive::new(
                        // source: objs/triangle.obj
                        Arc::new(
                            Shape::new_triangle(
                                p!(1, 0, 0),
                                p!(0, 1, 0),
                                // Co-ordinate system correction
                                p!(0, 0, -1),
                            )
                            .unwrap()
                        ),
                        Arc::new(Material::new_matte(Color::WHITE, 0.0)),
                    ))
                ],
            )
        );
    }
}
