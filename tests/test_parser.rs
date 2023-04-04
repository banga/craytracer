#[cfg(test)]
mod tokenizer {
    use craytracer::scene_parser::tokenizer::{tokenize, ParserError, Token};
    use pretty_assertions::assert_eq;

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

#[cfg(test)]
mod parser {
    use pretty_assertions::assert_eq;
    use std::{collections::HashMap, sync::Arc};

    use craytracer::{
        camera::Camera,
        color::Color,
        material::Material,
        primitive::Primitive,
        scene::Scene,
        scene_parser::parser::{
            parse_raw_value, RawValue, RawValueArray, RawValueMap, TypedRawValueMap,
        },
        scene_parser::scene_parser::parse_scene,
        scene_parser::tokenizer::{tokenize, ParserError},
        shape::Shape,
        vector::Vector,
    };

    fn expect_raw_value(s: &str, expected: RawValue) {
        let result = parse_raw_value(&mut tokenize(s).unwrap().iter().peekable())
            .expect(&format!("{:?} failed to parse", s));
        assert_eq!(result, expected, "{:?} parsed to unexpected value", s);
    }

    fn expect_raw_value_map(s: &str, expected: HashMap<String, RawValue>) {
        expect_raw_value(s, RawValue::Map(RawValueMap { map: expected }));
    }

    fn expect_raw_value_array(s: &str, expected: Vec<RawValue>) {
        expect_raw_value(s, RawValue::Array(RawValueArray { array: expected }));
    }

    fn expect_parse_error(s: &str, error: &str) {
        let result = parse_raw_value(&mut tokenize(s).unwrap().iter().peekable())
            .expect_err(&format!("{:?} parsed succesfully", s));
        assert_eq!(
            result,
            ParserError {
                message: error.to_string()
            },
        );
    }

    #[test]
    fn raw_value() {
        // TODO: Fix trailing comma parsing. We currently allow skipping the comma in many cases
        expect_raw_value("1.23", RawValue::Number(1.23));
        expect_raw_value("'hello'", RawValue::String("hello".to_string()));
        expect_raw_value(
            "Vector(1, -2, 3.1)",
            RawValue::Vector(Vector(1.0, -2.0, 3.1)),
        );
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
            }),
        );
        expect_raw_value(
            "{ x: 1, y: 'z' }",
            RawValue::Map(RawValueMap {
                map: HashMap::from([
                    ("x".to_string(), RawValue::Number(1.0)),
                    ("y".to_string(), RawValue::String("z".to_string())),
                ]),
            }),
        );
        expect_raw_value(
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
                    }),
                ],
            }),
        );

        expect_parse_error("x", "Expected LeftBrace, got Eof");
        expect_parse_error(",", "Expected a raw value. Got Comma");
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
        );

        expect_parse_error("{ x: 1, x: 2 }", "Duplicate key x");
        expect_parse_error("{ x: 1", "Expected RightBrace, got Eof");
        expect_parse_error("{ x 1 }", "Expected Colon, got Number(1.0)");
        expect_parse_error("{ 1: x }", "Expected RightBrace, got Number(1.0)");
        expect_parse_error(
            "{ x: 1 y: 2 }",
            "Expected RightBrace, got Identifier(\"y\")",
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
                }),
            ],
        );

        expect_parse_error("[", "Expected a raw value. Got Eof");
        expect_parse_error("[,]", "Expected a raw value. Got Comma");
        expect_parse_error("[1 2]", "Expected RightBracket, got Number(2.0)");
    }

    #[test]
    fn scene() {
        assert_eq!(
            parse_scene(
                "
{
    max_depth: 5,
    camera: Projection {
        origin: Vector(0, 0, 0),
        target: Vector(0, 0, 1),
        up: Vector(0, 1, 0),
        focal_distance: 1,
        film_width: 400,
        film_height: 300,
        num_samples: 1
    },
    materials: {
        ball: Matte {
            reflectance: Color(1, 1, 1),
            sigma: 0
        },
    },
    shapes: {
        ball: Sphere {
            origin: Vector(0, 0, 2),
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
                5,
                400,
                300,
                Box::new(Camera::new_projection_camera(
                    Vector::O,
                    Vector::Z,
                    Vector::Y,
                    1.0,
                    1,
                    400,
                    300
                )),
                vec![
                    Arc::new(Primitive::new_shape_primitive(
                        Arc::new(Shape::new_sphere(Vector(0.0, 0.0, 2.0), 1.0)),
                        Arc::new(Material::new_matte(Color::WHITE, 0.0))
                    )),
                    Arc::new(Primitive::new_shape_primitive(
                        // source: objs/triangle.obj
                        Arc::new(Shape::new_triangle(
                            Vector(1.0, 0.0, 0.0),
                            Vector(0.0, 1.0, 0.0),
                            Vector(0.0, 0.0, 1.0),
                        )),
                        Arc::new(Material::new_matte(Color::WHITE, 0.0))
                    ))
                ],
            )
        );
    }
}
