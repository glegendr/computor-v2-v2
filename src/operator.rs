#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    Var(String),
    Number { number: f64, x: i32, i: i32 },
    Mat(Vec<Vec<Operator>>),
    Add,
    Minus,
    Mult,
    MatricialMult,
    Modulo,
    Power,
    OpenParenthesis,
    CloseParenthesis,
    Div,
    Equal
}

fn i_mult(i: &i32) -> f64 {
    if i % 4 < 2 {
        1.
    }  else {
        -1.
    }
}

impl Operator {
    pub fn get_precedence(&self) -> u8 {
        match self {
            Operator::Add | Operator::Minus => 2,
            Operator::Mult | Operator::Div => 3,
            Operator::Power => 4,
            Operator::CloseParenthesis | Operator::OpenParenthesis => 1,
            _ => unreachable!()
        }
    }

    pub fn get_associativity(&self) -> bool {
        match self {
            Operator::Power => true,
            _ => false
        }
    }

    pub fn from_str(value: &str) -> Result<Self, String> {
        match value.trim() {
            "+" => Ok(Self::Add),
            "-" => Ok(Self::Minus),
            "*" => Ok(Self::Mult),
            "**" => Ok(Self::MatricialMult),
            "%" => Ok(Self::Modulo),
            "^" => Ok(Self::Power),
            "(" => Ok(Self::OpenParenthesis),
            ")" => Ok(Self::CloseParenthesis),
            "/" => Ok(Self::Div),
            "=" => Ok(Self::Equal),
            value => {
                if let Some(ope) = Operator::parse_mat(value) {
                    return Ok(ope)
                }
                match value.parse::<f64>() {
                    Ok(value_nb) => Ok(Self::Number { number: value_nb, x: 0, i: 0 }),
                    _ => {
                        if value.contains(|c: char| "+-*%^,; .[]/".contains(c) || String::from(c).parse::<i32>().is_ok()) {
                            Err(format!("{value} is invalid name for a variable"))
                        } else {
                            Ok(Self::Var(String::from(value)))
                        }
                    }
                }
            }
        }
    }

    fn parse_mat(value: &str) -> Option<Self> {
        let mut mat: Vec<Vec<Operator>> = Vec::default();
        let mut depth = 0;
        let mut mat_id  = 0;
        let mut comma_id = 0;
        for c in value.split_inclusive(|c| ";[],".contains(c)) {
            match c.trim() {
                "[" => depth += 1,
                "]" => {
                    mat_id += 1;
                    depth -= 1
                },
                ";" => {
                    if depth != 1 {
                        return None
                    }
                    comma_id += 1
                },
                _ => {
                    let mut string = String::from(c);
                    if let Some(last) = string.pop() {
                        if !string.trim().is_empty() {
                            if let Ok(nb) = string.trim().parse::<f64>() {
                                if depth != 2 || mat_id != comma_id {
                                    return None
                                }
                                if let Some(row) = mat.get_mut(mat_id) {
                                    row.push(Operator::Number { number: nb, x: 0, i: 0 })
                                } else {
                                    mat.push(vec![Operator::Number { number: nb, x: 0, i: 0 }])
                                }
                            } else if let Ok(ope) = Operator::from_str(&string) {
                                if depth != 2 || mat_id != comma_id {
                                    return None
                                }
                                if let Some(row) = mat.get_mut(mat_id) {
                                    row.push(ope)
                                } else {
                                    mat.push(vec![ope])
                                }
                            } else {
                                return None
                            }
                            match last {
                                ']' => {
                                    mat_id += 1;
                                    depth -= 1
                                },
                                ';' | '[' => {
                                    return None
                                },
                                _ => {}
                            }
                        } else if let Ok(nb) = string.trim().parse::<f64>() {
                            println!("{:?}", Operator::Number { number: nb, x: 0, i: 0 });
                        }
                    }
                }
            };
            if depth < 0 ||  depth > 2 {
                return None
            }
        }
        if depth != 0 {
            return None
        }
        if mat.is_empty() || mat.iter().fold((false, -1 ), |acc, row| (acc.0 || (row.len() as i32 != acc.1 && acc.1 != -1), row.len() as i32)).0 {
            return None
        }
        Some(Operator::Mat(mat))
    }

    pub fn calc(&self, a: &Operator, b: &Operator) -> Option<Operator> {
        match self {
            Self::Add => {
                match (a, b) {
                    (Operator::Number { number, x, i }, Operator::Number { number: number_b, x: x_b, i: i_b }) => {
                        if x == x_b && i % 2 == i_b % 2 {
                            return Some(Operator::Number { number: number * i_mult(i) + number_b * i_mult(i_b), x: *x, i: i % 2 })
                        } else if number == &0. {
                            return Some(Operator::Number { number: number_b * i_mult(i_b), x: *x_b, i: i_b % 2 })
                        } else if number_b == &0. {
                            return Some(Operator::Number { number: number * i_mult(i), x: *x, i: i % 2 })
                        }
                        None
                    },
                    (Operator::Mat(mat), Operator::Mat(mat_b)) => {
                        if mat.len() != mat_b.len() || mat.get(0).is_none() || mat_b.get(0).is_none() || mat.get(0).unwrap().len() != mat_b.get(0).unwrap().len() {
                            return None   
                        }
                        let mut new_mat = Vec::with_capacity(mat.len());
                        let mut row_id = 0;
                        for row in mat {
                            let mut new_row = Vec::with_capacity(row.len());
                            let mut  ope_id = 0;
                            for ope in row {
                                match mat_b.get(row_id) {
                                    Some(row_b) => {
                                        match (ope, row_b.get(ope_id)) {
                                            (a@Self::Number {..}, Some(b@Self::Number {..})) => {
                                                match Self::Add.calc(a, b) {
                                                    Some(x) => new_row.push(x),
                                                    _ => return None
                                                }
                                            },
                                            _ => return None
                                        }
                                    },
                                    None => return None
                                }
                                ope_id += 1;
                            }
                            row_id += 1;
                            new_mat.push(new_row);
                        }
                        Some(Self::Mat(new_mat))
                    }
                    (Operator::Number { number, x, i }, Operator::Mat(mat)) | (Operator::Mat(mat), Operator::Number { number, x, i }) => {
                        if *x == 0 && i % 2 == 0 {
                            return Some(Operator::Mat(mat
                                .iter()
                                .map(|row| row
                                    .iter()
                                    .map(|ope| if let Operator::Number {number: number_b, ..} = ope {
                                        Operator::Number { number: number * i_mult(i) + number_b, x: 0, i: 0 }
                                    } else {
                                        Operator::Number { number: number * i_mult(i), x: 0, i: 0 }
                                    })
                                    .collect()
                                ).collect()));
                        }
                        None
                    }
                    _ => None,
                }
            }
            Self::Minus => {
                match (a, b) {
                    (Operator::Number { number, x, i }, Operator::Number { number: number_b, x: x_b, i: i_b }) => {
                        if x == x_b && i % 2 == i_b % 2 {
                            return Some(Operator::Number { number: number * i_mult(i) - number_b * i_mult(i_b), x: *x, i: i % 2 })
                        } else if number == &0. {
                            return Some(Operator::Number { number: -number_b * i_mult(i_b), x: *x_b, i: i_b % 2 })
                        } else if number_b == &0. {
                            return Some(Operator::Number { number: number * i_mult(i), x: *x, i: i % 2 })
                        }
                        None
                    }
                    (Operator::Number { x, i, .. }, Operator::Mat(mat)) => {
                        if *x == 0 && i % 2 == 0 {
                            let mut new_mat = Vec::with_capacity(mat.len());
                            for row in mat {
                                let mut new_row = Vec::with_capacity(row.len());
                                for ope in row {
                                    match Self::Minus.calc(a, ope) {
                                        Some(x) => new_row.push(x),
                                        _ => return None
                                    }
                                    
                                }
                                new_mat.push(new_row);
                            }
                            return Some(Self::Mat(new_mat))
                        }
                        None
                    }
                    (Operator::Mat(mat), Operator::Number { x, i, .. }) => {
                        if *x == 0 && i % 2 == 0 {
                            let mut new_mat = Vec::with_capacity(mat.len());
                            for row in mat {
                                let mut new_row = Vec::with_capacity(row.len());
                                for ope in row {
                                    match Self::Minus.calc(ope, b) {
                                        Some(x) => new_row.push(x),
                                        _ => return None
                                    }
                                    
                                }
                                new_mat.push(new_row);
                            }
                            return Some(Self::Mat(new_mat))
                        }
                        None
                    }
                    (Operator::Mat(mat), Operator::Mat(mat_b)) => {
                        if mat.len() != mat_b.len() || mat.get(0).is_none() || mat_b.get(0).is_none() || mat.get(0).unwrap().len() != mat_b.get(0).unwrap().len() {
                            return None   
                        }
                        let mut new_mat = Vec::with_capacity(mat.len());
                        let mut row_id = 0;
                        for row in mat {
                            let mut new_row = Vec::with_capacity(row.len());
                            let mut  ope_id = 0;
                            for ope in row {
                                match mat_b.get(row_id) {
                                    Some(row_b) => {
                                        match (ope, row_b.get(ope_id)) {
                                            (a@Self::Number {..}, Some(b@Self::Number {..})) => {
                                                match Self::Minus.calc(a, b) {
                                                    Some(x) => new_row.push(x),
                                                    _ => return None
                                                }
                                            },
                                            _ => return None
                                        }
                                    },
                                    None => return None
                                }
                                ope_id += 1;
                            }
                            row_id += 1;
                            new_mat.push(new_row);
                        }
                        Some(Self::Mat(new_mat))
                    }
                    _ => None,
                }
            },
            Self::Mult => {
                match (a, b) {
                    (Operator::Number { number, x, i }, Operator::Number { number: number_b, x: x_b, i: i_b }) => {
                        if *number == 0. || *number_b == 0. {
                            return Some(Self::Number { number: 0., x: 0, i: 0 })
                        }
                        Some(Self::Number { number: number * number_b * i_mult(&(i + i_b)), x: x + x_b, i: (i + i_b) % 2 })
                    }
                    (Operator::Mat(mat), Operator::Mat(mat_b)) => {
                        if mat.len() != mat_b.len() || mat.get(0).is_none() || mat_b.get(0).is_none() || mat.get(0).unwrap().len() != mat_b.get(0).unwrap().len() {
                            return None   
                        }
                        let mut new_mat = Vec::with_capacity(mat.len());
                        let mut row_id = 0;
                        for row in mat {
                            let mut new_row = Vec::with_capacity(row.len());
                            let mut  ope_id = 0;
                            for ope in row {
                                match mat_b.get(row_id) {
                                    Some(row_b) => {
                                        match (ope, row_b.get(ope_id)) {
                                            (a@Self::Number {..}, Some(b@Self::Number {..})) => {
                                                match Self::Mult.calc(a, b) {
                                                    Some(x) => new_row.push(x),
                                                    _ => return None
                                                }
                                            },
                                            _ => return None
                                        }
                                    },
                                    None => return None
                                }
                                ope_id += 1;
                            }
                            row_id += 1;
                            new_mat.push(new_row);
                        }
                        Some(Self::Mat(new_mat))
                    }
                    (Operator::Number { number, x, i }, Operator::Mat(mat)) | (Operator::Mat(mat), Operator::Number { number, x, i }) => {
                        if *x == 0 && i % 2 == 0 {
                            return Some(Operator::Mat(mat
                                .iter()
                                .map(|row| row
                                    .iter()
                                    .map(|ope| if let Operator::Number {number: number_b, ..} = ope {
                                        Operator::Number { number: number * i_mult(i) * number_b, x: 0, i: 0 }
                                    } else {
                                        Operator::Number { number: number * i_mult(i), x: 0, i: 0 }
                                    })
                                    .collect()
                                ).collect()));
                        }
                        None
                    }
                    _ => None,
                }
            }
            Self::Modulo => {
                match (a, b) {
                    (Operator::Number { number, x, i }, Operator::Number { number: number_b, x: x_b, i: i_b }) => {
                        if *x == 0 && *x_b == 0 && i % 2 == 0 && i_b % 2 == 0 && *number_b != 0. {
                            return Some(Self::Number { number: number % number_b, x: 0, i: 0 })
                        }
                        None
                    }
                    (Operator::Mat(mat), Operator::Mat(mat_b)) => {
                        if mat.len() != mat_b.len() || mat.get(0).is_none() || mat_b.get(0).is_none() || mat.get(0).unwrap().len() != mat_b.get(0).unwrap().len() {
                            return None   
                        }
                        let mut new_mat = Vec::with_capacity(mat.len());
                        let mut row_id = 0;
                        for row in mat {
                            let mut new_row = Vec::with_capacity(row.len());
                            let mut  ope_id = 0;
                            for ope in row {
                                match mat_b.get(row_id) {
                                    Some(row_b) => {
                                        match (ope, row_b.get(ope_id)) {
                                            (a@Self::Number {..}, Some(b@Self::Number {..})) => {
                                                match Self::Modulo.calc(a, b) {
                                                    Some(x) => new_row.push(x),
                                                    _ => return None
                                                }
                                            },
                                            _ => return None
                                        }
                                    },
                                    None => return None
                                }
                                ope_id += 1;
                            }
                            row_id += 1;
                            new_mat.push(new_row);
                        }
                        Some(Self::Mat(new_mat))
                    }
                    (a@Operator::Number { x, i, .. }, Operator::Mat(mat)) => {
                        if *x == 0 && i % 2 == 0 {
                            let mut new_mat = Vec::with_capacity(mat.len());
                            for row in mat {
                                let mut new_row = Vec::with_capacity(row.len());
                                for ope in row {
                                    match Self::Modulo.calc(a, ope) {
                                        Some(x) => new_row.push(x),
                                        _ => return None
                                    }
                                    
                                }
                                new_mat.push(new_row);
                            }
                            return Some(Self::Mat(new_mat))
                        }
                        None
                    }
                    (Operator::Mat(mat), b@Operator::Number { x, i, .. }) => {
                        if *x == 0 && i % 2 == 0 {
                            let mut new_mat = Vec::with_capacity(mat.len());
                            for row in mat {
                                let mut new_row = Vec::with_capacity(row.len());
                                for ope in row {
                                    match Self::Modulo.calc(ope, b) {
                                        Some(x) => new_row.push(x),
                                        _ => return None
                                    }
                                    
                                }
                                new_mat.push(new_row);
                            }
                            return Some(Self::Mat(new_mat))
                        }
                        None
                    }
                    _ => None,
                }
            }
            Self::MatricialMult => {
                match (a, b) {
                    (Operator::Mat(mat), Operator::Mat(mat_b)) => {
                        if mat.get(0).is_none() || mat.get(0).unwrap().len() != mat_b.len() {
                            return None   
                        }
                        let mut new_mat = Vec::new();
                            for row in mat {
                            let mut new_row = Vec::new();
                            for col_id in 0..mat_b.len() {
                                let mut acc = Self::Number { number: 0., x: 0, i: 0 };
                                for (i, row_b) in mat_b.iter().enumerate() {
                                    match Self::Mult.calc(row.get(i).unwrap(), row_b.get(col_id).unwrap()) {
                                        Some(mult) => match Self::Add.calc(&acc, &mult) {
                                            Some(new_acc) => acc = new_acc,
                                            None => return None
                                        },
                                        None => return None
                                    }
                                }
                                new_row.push(acc);
                            }
                            new_mat.push(new_row);
                        }
                        Some(Self::Mat(new_mat))
                    },
                    _ => None
                }
            }
            Self::Div => {
                match (a, b) {
                    (Operator::Number { number, x, i }, Operator::Number { number: number_b, x: x_b, i: i_b }) => {
                        if *number_b == 0. {
                            return None
                        }
                        Some(Self::Number { number: (number / number_b) * i_mult(&(i - i_b)), x: x - x_b, i: (i - i_b) % 2 })
                    }
                    (Operator::Mat(mat), Operator::Mat(mat_b)) => {
                        if mat.len() != mat_b.len() || mat.get(0).is_none() || mat_b.get(0).is_none() || mat.get(0).unwrap().len() != mat_b.get(0).unwrap().len() {
                            return None   
                        }
                        let mut new_mat = Vec::with_capacity(mat.len());
                        let mut row_id = 0;
                        for row in mat {
                            let mut new_row = Vec::with_capacity(row.len());
                            let mut  ope_id = 0;
                            for ope in row {
                                match mat_b.get(row_id) {
                                    Some(row_b) => {
                                        match (ope, row_b.get(ope_id)) {
                                            (a@Self::Number {..}, Some(b@Self::Number {..})) => {
                                                match Self::Div.calc(a, b) {
                                                    Some(x) => new_row.push(x),
                                                    _ => return None
                                                }
                                            },
                                            _ => return None
                                        }
                                    },
                                    None => return None
                                }
                                ope_id += 1;
                            }
                            row_id += 1;
                            new_mat.push(new_row);
                        }
                        Some(Self::Mat(new_mat))
                    }
                    (a@Operator::Number { x, i, .. }, Operator::Mat(mat)) => {
                        if *x == 0 && i % 2 == 0 {
                            let mut new_mat = Vec::with_capacity(mat.len());
                            for row in mat {
                                let mut new_row = Vec::with_capacity(row.len());
                                for ope in row {
                                    match Self::Div.calc(a, ope) {
                                        Some(x) => new_row.push(x),
                                        _ => return None
                                    }
                                    
                                }
                                new_mat.push(new_row);
                            }
                            return Some(Self::Mat(new_mat))
                        }
                        None
                    }
                    (Operator::Mat(mat), b@Operator::Number { x, i, .. }) => {
                        if *x == 0 && i % 2 == 0 {
                            let mut new_mat = Vec::with_capacity(mat.len());
                            for row in mat {
                                let mut new_row = Vec::with_capacity(row.len());
                                for ope in row {
                                    match Self::Div.calc(ope, b) {
                                        Some(x) => new_row.push(x),
                                        _ => return None
                                    }
                                    
                                }
                                new_mat.push(new_row);
                            }
                            return Some(Self::Mat(new_mat))
                        }
                        None
                    }
                    _ => None,
                }
            }
            Self::Power => {
                match (a, b) {
                    (Self::Number { number, x, i }, Self::Number { number: nb_b, x: x_b, i: i_b }) => {
                        if *nb_b == 0. {
                            return Some(Self::Number { number: 1., x: 0, i: 0 })
                        } else if i_b % 2 != 0  || *x_b != 0 {
                            return None
                        }
                        Some(Self::Number { number: number.powf(*nb_b) * i_mult(&(i * i_b)), x: x * x_b, i: (i * i_b) % 2 })
                    }
                    (a@Operator::Number { x, i, .. }, Operator::Mat(mat)) => {
                        if *x == 0 && i % 2 == 0 {
                            let mut new_mat = Vec::with_capacity(mat.len());
                            for row in mat {
                                let mut new_row = Vec::with_capacity(row.len());
                                for ope in row {
                                    match Self::Power.calc(a, ope) {
                                        Some(x) => new_row.push(x),
                                        _ => return None
                                    }
                                    
                                }
                                new_mat.push(new_row);
                            }
                            return Some(Self::Mat(new_mat))
                        }
                        None
                    }
                    (Operator::Mat(_), Operator::Number { x, i, number }) => {
                        if *x == 0 && i % 2 == 0 {
                            let mut acc = a.clone();
                            for _ in 1..number.round() as i64 {
                                match Self::Power.calc(&acc, a) {
                                    Some(new_acc) => acc = new_acc,
                                    _ => return None
                                }
                            }
                            return Some(acc)
                        }
                        None
                    }
                    _ => None
                }
            },
            _ => None,
        }
    }
}


mod matrices {
    use super::*;

    #[test]
    fn good_mat() {
        assert_eq!(Operator::parse_mat("[[1 ,2  ,3   ];[ 1, 2,3 ]]"), Some(Operator::Mat(vec![
            vec![
                Operator::Number {number: 1., x: 0, i: 0 },
                Operator::Number {number: 2., x: 0, i: 0 },
                Operator::Number {number: 3., x: 0, i: 0 }
            ], vec![
                Operator::Number {number: 1., x: 0, i: 0 },
                Operator::Number {number: 2., x: 0, i: 0 },
                Operator::Number {number: 3., x: 0, i: 0 }
            ]
            ])));
            assert_eq!(Operator::parse_mat("[[                 1,2]]"), Some(Operator::Mat(vec![
                vec![
                    Operator::Number {number: 1., x: 0, i: 0 },
                    Operator::Number {number: 2., x: 0, i: 0 }
                ]
                ])));
            assert_eq!(Operator::parse_mat("[[\t\t\nsalut,2]]"), Some(Operator::Mat(vec![
                vec![
                    Operator::Var(String::from("salut")),
                    Operator::Number {number: 2., x: 0, i: 0 }
                ]
                ])));
            assert_eq!(Operator::parse_mat("[[\t\t\n1,2]]"), Some(Operator::Mat(vec![
                vec![
                    Operator::Number {number: 1., x: 0, i: 0 },
                    Operator::Number {number: 2., x: 0, i: 0 }
                ]
                ])));
            assert_eq!(Operator::parse_mat("[[1];[2];[3];[4];[5]]"), Some(Operator::Mat(vec![
                vec![Operator::Number {number: 1., x: 0, i: 0 }],
                vec![Operator::Number {number: 2., x: 0, i: 0 }],
                vec![Operator::Number {number: 3., x: 0, i: 0 }],
                vec![Operator::Number {number: 4., x: 0, i: 0 }],
                vec![Operator::Number {number: 5., x: 0, i: 0 }],
                ])));
    }
    
    #[test]
    fn bad_size() {
        assert_eq!(Operator::parse_mat("[[1,2,3];[1,2]]"), None);
        assert_eq!(Operator::parse_mat("[[1,2,3];[1,2,3, 4];[1,2, 3]]"), None);
        assert_eq!(Operator::parse_mat("[[1,2,3];[];[2,3, 4];[1,2, 3]]"), None);
        assert_eq!(Operator::parse_mat("[[1];[2];[3];[4];[]]]"), None);
        assert_eq!(Operator::parse_mat("[[1];[2];[3];[4];[5, 6]]]"), None);
    }

    #[test]
    fn double_point_virgule() {
        assert_eq!(Operator::parse_mat("[[1,2,3];;[1,2, 3]]"), None);
        assert_eq!(Operator::parse_mat("[;[1,2,3][1,2, 3]]"), None);
        assert_eq!(Operator::parse_mat("[[1,2,3][1,2, 3];]"), None);
        assert_eq!(Operator::parse_mat("[[2,3,4;][1,2, 3]]"), None);
        assert_eq!(Operator::parse_mat("[[3,4,5][;1,2, 3]]"), None);
    }

    #[test]
    fn no_point_virgule() {
        assert_eq!(Operator::parse_mat("[[1,2,3][1,2, 3]]"), None);
        assert_eq!(Operator::parse_mat("[[1,2,3];[1,2,3][1,2, 3]]"), None);
    }

    #[test]
    fn bad_depth() {
        assert_eq!(Operator::parse_mat("[[1,2,3];[[],2,3, 4];[1,2, 3]]"), None);
        assert_eq!(Operator::parse_mat("[1,2,3]"), None);
    }

    #[test]
    fn operator_inside() {
        assert_eq!(Operator::parse_mat("[[1,2,3];[2,3 + 2, 4];[1,2, 3]]"), None);
        assert_eq!(Operator::parse_mat("[[1,2,3];[2,3 * 2, 4];[1,2, 3]]"), None);
        assert_eq!(Operator::parse_mat("[[1,2,3];[2,3 / 2, 4];[1,2, 3]]"), None);
        assert_eq!(Operator::parse_mat("[[1,2,3];[2,3 % 2, 4];[1,2, 3]]"), None);
        assert_eq!(Operator::parse_mat("[[1,2,3];[2,3 ^ 2, 4];[1,2, 3]]"), None);
    }

    #[test]
    fn bad_separator() {
        assert_eq!(Operator::parse_mat("[[1,2,3],[2,3, 4],[1,2, 3]]"), None);
        assert_eq!(Operator::parse_mat("[[1,2,3];[2;3, 4];[1,2, 3]]"), None);
        assert_eq!(Operator::parse_mat("[[1,2,3];[2.3 . 4];[1,2, 3]]"), None);
        assert_eq!(Operator::parse_mat("[[1,2,3];[2 # 3 # 4];[1,2, 3]]"), None);
        assert_eq!(Operator::parse_mat("[[1,2,3];[2 ! 3 ! 4];[1,2, 3]]"), None);
        assert_eq!(Operator::parse_mat("[[1,2,3];[2 \n 3 , 4];[1,2, 3]]"), None);
    }

    #[test]
    fn empty() {
        assert_eq!(Operator::parse_mat(""), None);
        assert_eq!(Operator::parse_mat("[]"), None);
        assert_eq!(Operator::parse_mat("[[]]"), None);
        assert_eq!(Operator::parse_mat("[[];[]]"), None);
        assert_eq!(Operator::parse_mat("[[];[];[]]"), None);
    }

}

mod add {

    use super::*;

    #[test]
    fn add_simple() {
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 0., x: 0, i: 0}, &Operator::Number { number: 4., x: 0, i: 0}), Some(Operator::Number { number: 4., x: 0, i: 0}));
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 3., x: 0, i: 0}, &Operator::Number { number: 2., x: 0, i: 0}), Some(Operator::Number { number: 5., x: 0, i: 0}));
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 3., x: 0, i: 0}, &Operator::Number { number: -2., x: 0, i: 0}), Some(Operator::Number { number: 1., x: 0, i: 0}));
        assert_eq!(Operator::Add.calc(&Operator::Number { number: -3., x: 0, i: 0}, &Operator::Number { number: -2., x: 0, i: 0}), Some(Operator::Number { number: -5., x: 0, i: 0}));
        assert_eq!(Operator::Add.calc(&Operator::Number { number: -3., x: 0, i: 0}, &Operator::Number { number: 1., x: 0, i: 0}), Some(Operator::Number { number: -2., x: 0, i: 0}));
    }

    #[test]
    fn add_x() {
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 3., x: 1, i: 0}, &Operator::Number { number: 4., x: 1, i: 0}), Some(Operator::Number { number: 7., x: 1, i: 0}));
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 3., x: 2, i: 0}, &Operator::Number { number: 4., x: 2, i: 0}), Some(Operator::Number { number: 7., x: 2, i: 0}));
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 3., x: 2, i: 0}, &Operator::Number { number: -4., x: 2, i: 0}), Some(Operator::Number { number: -1., x: 2, i: 0}));
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 3., x: 99, i: 0}, &Operator::Number { number: -4., x: 99, i: 0}), Some(Operator::Number { number: -1., x: 99, i: 0}));
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 3., x: 99, i: 0}, &Operator::Number { number: -4., x: 1, i: 0}), None);
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 3., x: 99, i: 0}, &Operator::Number { number: -4., x: 98, i: 0}), None);
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 3., x: 99, i: 0}, &Operator::Number { number: 3., x: 98, i: 0}), None);
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 3., x: -1, i: 0}, &Operator::Number { number: 3., x: -2, i: 0}), None);
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 3., x: -1, i: 0}, &Operator::Number { number: 3., x: -1, i: 0}), Some(Operator::Number { number: 6., x: -1, i: 0}));
    }

    #[test]
    fn add_i() {
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 3., x: 0, i: 1}, &Operator::Number { number: 4., x: 0, i: 1}), Some(Operator::Number { number: 7., x: 0, i: 1}));
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 3., x: 0, i: 1}, &Operator::Number { number: 4., x: 0, i: 2}), None);
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 3., x: 0, i: 0}, &Operator::Number { number: 4., x: 0, i: 2}), Some(Operator::Number { number: -1., x: 0, i: 0}));
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 3., x: 0, i: 0}, &Operator::Number { number: 4., x: 0, i: 98}), Some(Operator::Number { number: -1., x: 0, i: 0}));
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 3., x: 0, i: 1}, &Operator::Number { number: 4., x: 0, i: 3}), Some(Operator::Number { number: -1., x: 0, i: 1}));
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 0., x: 0, i: 1}, &Operator::Number { number: 4., x: 0, i: 3}), Some(Operator::Number { number: -4., x: 0, i: 1}));
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 0., x: 0, i: 0}, &Operator::Number { number: 4., x: 0, i: 3}), Some(Operator::Number { number: -4., x: 0, i: 1}));
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 4., x: 0, i: 3}, &Operator::Number { number: 0., x: 0, i: 3}), Some(Operator::Number { number: -4., x: 0, i: 1}));
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 4., x: 0, i: 3}, &Operator::Number { number: 0., x: 0, i: 0}), Some(Operator::Number { number: -4., x: 0, i: 1}));
    }

    #[test]
    fn add_i_x() {
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 3., x: 1, i: 1}, &Operator::Number { number: 4., x: 1, i: 3}), Some(Operator::Number { number: -1., x: 1, i: 1}));
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 3., x: 1, i: 1}, &Operator::Number { number: 4., x: 0, i: 3}), None);
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 3., x: 2, i: 1}, &Operator::Number { number: 4., x: 2, i: 3}), Some(Operator::Number { number: -1., x: 2, i: 1}));
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 3., x: 2, i: 2}, &Operator::Number { number: 4., x: 2, i: 0}), Some(Operator::Number { number: 1., x: 2, i: 0}));
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 3., x: 2, i: 4}, &Operator::Number { number: 4., x: 2, i: 0}), Some(Operator::Number { number: 7., x: 2, i: 0}));
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 3., x: 2, i: 4}, &Operator::Number { number: 4., x: 2, i: 4}), Some(Operator::Number { number: 7., x: 2, i: 0}));
    }

    #[test]
    fn add_nb_mat() {
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 3., x: 0, i: 2}, &Operator::Mat(vec![
            vec![Operator::Number { number: 1., x: 0, i: 0}, Operator::Number { number: 2., x: 0, i: 0}],
            vec![Operator::Number { number: 3., x: 0, i: 0}, Operator::Number { number: 4., x: 0, i: 0}]
        ])), Some(Operator::Mat(vec![
            vec![Operator::Number { number: -2., x: 0, i: 0}, Operator::Number { number: -1., x: 0, i: 0}],
            vec![Operator::Number { number: 0., x: 0, i: 0}, Operator::Number { number: 1., x: 0, i: 0}]
        ])));
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 3., x: 0, i: 3}, &Operator::Mat(vec![
            vec![Operator::Number { number: 1., x: 0, i: 0}, Operator::Number { number: 2., x: 0, i: 0}],
            vec![Operator::Number { number: 3., x: 0, i: 0}, Operator::Number { number: 4., x: 0, i: 0}]
        ])), None);
        assert_eq!(Operator::Add.calc(&Operator::Number { number: 3., x: 1, i: 0}, &Operator::Mat(vec![
            vec![Operator::Number { number: 1., x: 0, i: 0}, Operator::Number { number: 2., x: 0, i: 0}],
            vec![Operator::Number { number: 3., x: 0, i: 0}, Operator::Number { number: 4., x: 0, i: 0}]
        ])), None);
        assert_eq!(Operator::Add.calc(&Operator::Mat(vec![
            vec![Operator::Number { number: 1., x: 0, i: 0}, Operator::Number { number: 2., x: 0, i: 0}],
            vec![Operator::Number { number: 3., x: 0, i: 0}, Operator::Number { number: 4., x: 0, i: 0}]
        ]), &Operator::Number { number: 3., x: 0, i: 0}), Some(Operator::Mat(vec![
            vec![Operator::Number { number: 4., x: 0, i: 0}, Operator::Number { number: 5., x: 0, i: 0}],
            vec![Operator::Number { number: 6., x: 0, i: 0}, Operator::Number { number: 7., x: 0, i: 0}]
        ])));
        assert_eq!(Operator::Add.calc(&Operator::Mat(vec![
            vec![Operator::Number { number: 1., x: 0, i: 0}, Operator::Number { number: -42., x: 0, i: 0}, Operator::Number { number: 3., x: 0, i: 0}],
            vec![Operator::Number { number: 4., x: 0, i: 0}, Operator::Number { number: 13., x: 0, i: 0}, Operator::Number { number: 6., x: 0, i: 0}],
            vec![Operator::Number { number: 0., x: 0, i: 0}, Operator::Number { number: 8., x: 0, i: 0}, Operator::Number { number: 9., x: 0, i: 0}]
        ]), &Operator::Number { number: 42., x: 0, i: 0}), Some(Operator::Mat(vec![
            vec![Operator::Number { number: 43., x: 0, i: 0}, Operator::Number { number: 0., x: 0, i: 0}, Operator::Number { number: 45., x: 0, i: 0}],
            vec![Operator::Number { number: 46., x: 0, i: 0}, Operator::Number { number: 55., x: 0, i: 0}, Operator::Number { number: 48., x: 0, i: 0}],
            vec![Operator::Number { number: 42., x: 0, i: 0}, Operator::Number { number: 50., x: 0, i: 0}, Operator::Number { number: 51., x: 0, i: 0}]
        ])));

    }

    #[test]
    fn add_mat() {
        assert_eq!(Operator::Add.calc(&Operator::Mat(vec![
            vec![Operator::Number { number: 99., x: 0, i: 0}, Operator::Number { number: 98., x: 0, i: 0}],
            vec![Operator::Number { number: 97., x: 0, i: 0}, Operator::Number { number: 96., x: 0, i: 0}]
        ]), &Operator::Mat(vec![
            vec![Operator::Number { number: 1., x: 0, i: 0}, Operator::Number { number: 2., x: 0, i: 0}],
            vec![Operator::Number { number: 3., x: 0, i: 0}, Operator::Number { number: 4., x: 0, i: 0}]
        ])), Some(Operator::Mat(vec![
            vec![Operator::Number { number: 100., x: 0, i: 0}, Operator::Number { number: 100., x: 0, i: 0}],
            vec![Operator::Number { number: 100., x: 0, i: 0}, Operator::Number { number: 100., x: 0, i: 0}]
        ])));
        assert_eq!(Operator::Add.calc(&Operator::Mat(vec![
            vec![Operator::Number { number: 1., x: 0, i: 0}, Operator::Number { number: 213., x: 0, i: 0}, Operator::Number { number: 0., x: 0, i: 0}],
            vec![Operator::Number { number: -9., x: 0, i: 0}, Operator::Number { number: -92., x: 0, i: 0}, Operator::Number { number: -24., x: 0, i: 0}]
        ]), &Operator::Mat(vec![
            vec![Operator::Number { number: 123., x: 0, i: 0}, Operator::Number { number: 22., x: 0, i: 0}, Operator::Number { number: -22., x: 0, i: 0}],
            vec![Operator::Number { number: 982., x: 0, i: 0}, Operator::Number { number: 41., x: 0, i: 0}, Operator::Number { number: -42., x: 0, i: 0}]
        ])), Some(Operator::Mat(vec![
            vec![Operator::Number { number: 124., x: 0, i: 0}, Operator::Number { number: 235., x: 0, i: 0}, Operator::Number { number: -22., x: 0, i: 0}],
            vec![Operator::Number { number: 973., x: 0, i: 0}, Operator::Number { number: -51., x: 0, i: 0}, Operator::Number { number: -66., x: 0, i: 0}]
        ])));
        assert_eq!(Operator::Add.calc(&Operator::Mat(vec![
            vec![Operator::Number { number: 1., x: 0, i: 0}, Operator::Number { number: 213., x: 0, i: 0}, Operator::Number { number: 0., x: 0, i: 0}],
            vec![Operator::Number { number: -9., x: 0, i: 0}, Operator::Number { number: -92., x: 0, i: 0}, Operator::Number { number: -24., x: 0, i: 0}]
        ]), &Operator::Mat(vec![
            vec![Operator::Number { number: 22., x: 0, i: 0}, Operator::Number { number: -22., x: 0, i: 0}],
            vec![Operator::Number { number: 982., x: 0, i: 0}, Operator::Number { number: 41., x: 0, i: 0}, Operator::Number { number: -42., x: 0, i: 0}]
        ])), None);
    }

}

mod mult {

    use super::*;
    
    #[test]
    fn mult_simple() {
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 0., x: 0, i: 0}, &Operator::Number { number: 4., x: 0, i: 0}), Some(Operator::Number { number: 0., x: 0, i: 0}));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 3., x: 0, i: 0}, &Operator::Number { number: 2., x: 0, i: 0}), Some(Operator::Number { number: 6., x: 0, i: 0}));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 3., x: 0, i: 0}, &Operator::Number { number: -2., x: 0, i: 0}), Some(Operator::Number { number: -6., x: 0, i: 0}));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: -3., x: 0, i: 0}, &Operator::Number { number: -2., x: 0, i: 0}), Some(Operator::Number { number: 6., x: 0, i: 0}));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: -3., x: 0, i: 0}, &Operator::Number { number: 1., x: 0, i: 0}), Some(Operator::Number { number: -3., x: 0, i: 0}));
    }

    #[test]
    fn mult_x() {
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 3., x: 1, i: 0}, &Operator::Number { number: 4., x: 1, i: 0}), Some(Operator::Number { number: 12., x: 2, i: 0}));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 3., x: 2, i: 0}, &Operator::Number { number: 4., x: 2, i: 0}), Some(Operator::Number { number: 12., x: 4, i: 0}));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 3., x: 2, i: 0}, &Operator::Number { number: -4., x: 2, i: 0}), Some(Operator::Number { number: -12., x: 4, i: 0}));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 3., x: 99, i: 0}, &Operator::Number { number: -4., x: 99, i: 0}), Some(Operator::Number { number: -12., x: 198, i: 0}));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 3., x: 99, i: 0}, &Operator::Number { number: -4., x: 1, i: 0}), Some(Operator::Number { number: -12., x: 100, i: 0}));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 3., x: 99, i: 0}, &Operator::Number { number: -4., x: 98, i: 0}), Some(Operator::Number { number: -12., x: 197, i: 0}));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 3., x: 99, i: 0}, &Operator::Number { number: 3., x: 98, i: 0}), Some(Operator::Number { number: 9., x: 197, i: 0}));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 3., x: -1, i: 0}, &Operator::Number { number: 3., x: -2, i: 0}), Some(Operator::Number { number: 9., x: -3, i: 0}));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 3., x: -1, i: 0}, &Operator::Number { number: 3., x: -1, i: 0}), Some(Operator::Number { number: 9., x: -2, i: 0}));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 0., x: 1, i: 0}, &Operator::Number { number: 3., x: 2, i: 0}), Some(Operator::Number { number: 0., x: 0, i: 0}));
    }

    #[test]
    fn mult_i() {
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 3., x: 0, i: 1}, &Operator::Number { number: 4., x: 0, i: 1}), Some(Operator::Number { number: -12., x: 0, i: 0}));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 3., x: 0, i: 1}, &Operator::Number { number: 4., x: 0, i: 2}), Some(Operator::Number { number: -12., x: 0, i: 1}));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 3., x: 0, i: 0}, &Operator::Number { number: 4., x: 0, i: 2}), Some(Operator::Number { number: -12., x: 0, i: 0}));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 3., x: 0, i: 0}, &Operator::Number { number: 4., x: 0, i: 98}), Some(Operator::Number { number: -12., x: 0, i: 0}));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 3., x: 0, i: 1}, &Operator::Number { number: 4., x: 0, i: 3}), Some(Operator::Number { number: 12., x: 0, i: 0}));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 0., x: 0, i: 1}, &Operator::Number { number: 4., x: 0, i: 3}), Some(Operator::Number { number: 0., x: 0, i: 0}));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 0., x: 0, i: 0}, &Operator::Number { number: 4., x: 0, i: 3}), Some(Operator::Number { number: 0., x: 0, i: 0}));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 4., x: 0, i: 3}, &Operator::Number { number: 0., x: 0, i: 3}), Some(Operator::Number { number: 0., x: 0, i: 0}));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 4., x: 0, i: 3}, &Operator::Number { number: 0., x: 0, i: 0}), Some(Operator::Number { number: 0., x: 0, i: 0}));
    }

    #[test]
    fn mult_i_x() {
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 3., x: 1, i: 1}, &Operator::Number { number: 4., x: 1, i: 3}), Some(Operator::Number { number: 12., x: 2, i: 0}));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 3., x: 1, i: 1}, &Operator::Number { number: 4., x: 0, i: 3}), Some(Operator::Number { number: 12., x: 1, i: 0}));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 3., x: 2, i: 1}, &Operator::Number { number: 4., x: 2, i: 3}), Some(Operator::Number { number: 12., x: 4, i: 0}));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 3., x: 2, i: 2}, &Operator::Number { number: 4., x: 2, i: 0}), Some(Operator::Number { number: -12., x: 4, i: 0}));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 3., x: 2, i: 4}, &Operator::Number { number: 4., x: 2, i: 0}), Some(Operator::Number { number: 12., x: 4, i: 0}));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 3., x: 2, i: 4}, &Operator::Number { number: 4., x: 2, i: 4}), Some(Operator::Number { number: 12., x: 4, i: 0}));
    }

    #[test]
    fn mult_nb_mat() {
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 3., x: 0, i: 2}, &Operator::Mat(vec![
            vec![Operator::Number { number: 1., x: 0, i: 0}, Operator::Number { number: 2., x: 0, i: 0}],
            vec![Operator::Number { number: 3., x: 0, i: 0}, Operator::Number { number: 4., x: 0, i: 0}]
        ])), Some(Operator::Mat(vec![
            vec![Operator::Number { number: -3., x: 0, i: 0}, Operator::Number { number: -6., x: 0, i: 0}],
            vec![Operator::Number { number: -9., x: 0, i: 0}, Operator::Number { number: -12., x: 0, i: 0}]
        ])));
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 3., x: 0, i: 3}, &Operator::Mat(vec![
            vec![Operator::Number { number: 1., x: 0, i: 0}, Operator::Number { number: 2., x: 0, i: 0}],
            vec![Operator::Number { number: 3., x: 0, i: 0}, Operator::Number { number: 4., x: 0, i: 0}]
        ])), None);
        assert_eq!(Operator::Mult.calc(&Operator::Number { number: 3., x: 1, i: 0}, &Operator::Mat(vec![
            vec![Operator::Number { number: 1., x: 0, i: 0}, Operator::Number { number: 2., x: 0, i: 0}],
            vec![Operator::Number { number: 3., x: 0, i: 0}, Operator::Number { number: 4., x: 0, i: 0}]
        ])), None);
        assert_eq!(Operator::Mult.calc(&Operator::Mat(vec![
            vec![Operator::Number { number: 1., x: 0, i: 0}, Operator::Number { number: 2., x: 0, i: 0}],
            vec![Operator::Number { number: 3., x: 0, i: 0}, Operator::Number { number: 4., x: 0, i: 0}]
        ]), &Operator::Number { number: 3., x: 0, i: 0}), Some(Operator::Mat(vec![
            vec![Operator::Number { number: 3., x: 0, i: 0}, Operator::Number { number: 6., x: 0, i: 0}],
            vec![Operator::Number { number: 9., x: 0, i: 0}, Operator::Number { number: 12., x: 0, i: 0}]
        ])));
        assert_eq!(Operator::Mult.calc(&Operator::Mat(vec![
            vec![Operator::Number { number: 1., x: 0, i: 0}, Operator::Number { number: -42., x: 0, i: 0}, Operator::Number { number: 3., x: 0, i: 0}],
            vec![Operator::Number { number: 4., x: 0, i: 0}, Operator::Number { number: 13., x: 0, i: 0}, Operator::Number { number: 6., x: 0, i: 0}],
            vec![Operator::Number { number: 0., x: 0, i: 0}, Operator::Number { number: 8., x: 0, i: 0}, Operator::Number { number: 9., x: 0, i: 0}]
        ]), &Operator::Number { number: 42., x: 0, i: 0}), Some(Operator::Mat(vec![
            vec![Operator::Number { number: 42., x: 0, i: 0}, Operator::Number { number: -1764., x: 0, i: 0}, Operator::Number { number: 126., x: 0, i: 0}],
            vec![Operator::Number { number: 168., x: 0, i: 0}, Operator::Number { number: 546., x: 0, i: 0}, Operator::Number { number: 252., x: 0, i: 0}],
            vec![Operator::Number { number: 0., x: 0, i: 0}, Operator::Number { number: 336., x: 0, i: 0}, Operator::Number { number: 378., x: 0, i: 0}]
        ])));
        assert_eq!(Operator::Mult.calc(&Operator::Mat(vec![
            vec![Operator::Number { number: 1., x: 0, i: 0}, Operator::Number { number: -42., x: 0, i: 0}, Operator::Number { number: 3., x: 0, i: 0}],
            vec![Operator::Number { number: 4., x: 0, i: 0}, Operator::Number { number: 13., x: 0, i: 0}, Operator::Number { number: 6., x: 0, i: 0}],
            vec![Operator::Number { number: 0., x: 0, i: 0}, Operator::Number { number: 8., x: 0, i: 0}, Operator::Number { number: 9., x: 0, i: 0}]
        ]), &Operator::Number { number: 0., x: 0, i: 0}), Some(Operator::Mat(vec![
            vec![Operator::Number { number: 0., x: 0, i: 0}, Operator::Number { number: 0., x: 0, i: 0}, Operator::Number { number: 0., x: 0, i: 0}],
            vec![Operator::Number { number: 0., x: 0, i: 0}, Operator::Number { number: 0., x: 0, i: 0}, Operator::Number { number: 0., x: 0, i: 0}],
            vec![Operator::Number { number: 0., x: 0, i: 0}, Operator::Number { number: 0., x: 0, i: 0}, Operator::Number { number: 0., x: 0, i: 0}]
        ])));

    }

    #[test]
    fn mult_mat() {
        assert_eq!(Operator::Mult.calc(&Operator::Mat(vec![
            vec![Operator::Number { number: 99., x: 0, i: 0}, Operator::Number { number: 98., x: 0, i: 0}],
            vec![Operator::Number { number: 97., x: 0, i: 0}, Operator::Number { number: 96., x: 0, i: 0}]
        ]), &Operator::Mat(vec![
            vec![Operator::Number { number: 1., x: 0, i: 0}, Operator::Number { number: 2., x: 0, i: 0}],
            vec![Operator::Number { number: 3., x: 0, i: 0}, Operator::Number { number: 4., x: 0, i: 0}]
        ])), Some(Operator::Mat(vec![
            vec![Operator::Number { number: 99., x: 0, i: 0}, Operator::Number { number: 196., x: 0, i: 0}],
            vec![Operator::Number { number: 291., x: 0, i: 0}, Operator::Number { number: 384., x: 0, i: 0}]
        ])));
        assert_eq!(Operator::Mult.calc(&Operator::Mat(vec![
            vec![Operator::Number { number: 1., x: 0, i: 0}, Operator::Number { number: 213., x: 0, i: 0}, Operator::Number { number: 0., x: 0, i: 0}],
            vec![Operator::Number { number: -9., x: 0, i: 0}, Operator::Number { number: -92., x: 0, i: 0}, Operator::Number { number: -24., x: 0, i: 0}]
        ]), &Operator::Mat(vec![
            vec![Operator::Number { number: 123., x: 0, i: 0}, Operator::Number { number: 22., x: 0, i: 0}, Operator::Number { number: -22., x: 0, i: 0}],
            vec![Operator::Number { number: 982., x: 0, i: 0}, Operator::Number { number: 41., x: 0, i: 0}, Operator::Number { number: -42., x: 0, i: 0}]
        ])), Some(Operator::Mat(vec![
            vec![Operator::Number { number: 123., x: 0, i: 0}, Operator::Number { number: 4686., x: 0, i: 0}, Operator::Number { number: 0., x: 0, i: 0}],
            vec![Operator::Number { number: -8838., x: 0, i: 0}, Operator::Number { number: -3772., x: 0, i: 0}, Operator::Number { number: 1008., x: 0, i: 0}]
        ])));
        assert_eq!(Operator::Mult.calc(&Operator::Mat(vec![
            vec![Operator::Number { number: 1., x: 0, i: 0}, Operator::Number { number: 213., x: 0, i: 0}, Operator::Number { number: 0., x: 0, i: 0}],
            vec![Operator::Number { number: -9., x: 0, i: 0}, Operator::Number { number: -92., x: 0, i: 0}, Operator::Number { number: -24., x: 0, i: 0}]
        ]), &Operator::Mat(vec![
            vec![Operator::Number { number: 22., x: 0, i: 0}, Operator::Number { number: -22., x: 0, i: 0}],
            vec![Operator::Number { number: 982., x: 0, i: 0}, Operator::Number { number: 41., x: 0, i: 0}, Operator::Number { number: -42., x: 0, i: 0}]
        ])), None);
    }


}

mod sub {
    use super::*;

    #[test]
    fn sub_simple() {
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 0., x: 0, i: 0}, &Operator::Number { number: 4., x: 0, i: 0}), Some(Operator::Number { number: -4., x: 0, i: 0}));
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 3., x: 0, i: 0}, &Operator::Number { number: 2., x: 0, i: 0}), Some(Operator::Number { number: 1., x: 0, i: 0}));
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 3., x: 0, i: 0}, &Operator::Number { number: -2., x: 0, i: 0}), Some(Operator::Number { number: 5., x: 0, i: 0}));
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: -3., x: 0, i: 0}, &Operator::Number { number: -2., x: 0, i: 0}), Some(Operator::Number { number: -1., x: 0, i: 0}));
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: -3., x: 0, i: 0}, &Operator::Number { number: 1., x: 0, i: 0}), Some(Operator::Number { number: -4., x: 0, i: 0}));
    }

    #[test]
    fn sub_x() {
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 3., x: 1, i: 0}, &Operator::Number { number: 4., x: 1, i: 0}), Some(Operator::Number { number: -1., x: 1, i: 0}));
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 3., x: 2, i: 0}, &Operator::Number { number: 4., x: 2, i: 0}), Some(Operator::Number { number: -1., x: 2, i: 0}));
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 3., x: 2, i: 0}, &Operator::Number { number: -4., x: 2, i: 0}), Some(Operator::Number { number: 7., x: 2, i: 0}));
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 3., x: 99, i: 0}, &Operator::Number { number: -4., x: 99, i: 0}), Some(Operator::Number { number: 7., x: 99, i: 0}));
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 3., x: 99, i: 0}, &Operator::Number { number: -4., x: 1, i: 0}), None);
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 3., x: 99, i: 0}, &Operator::Number { number: -4., x: 98, i: 0}), None);
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 3., x: 99, i: 0}, &Operator::Number { number: 3., x: 98, i: 0}), None);
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 3., x: -1, i: 0}, &Operator::Number { number: 3., x: -2, i: 0}), None);
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 3., x: -1, i: 0}, &Operator::Number { number: 3., x: -1, i: 0}), Some(Operator::Number { number: 0., x: -1, i: 0}));
    }

    #[test]
    fn sub_i() {
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 3., x: 0, i: 1}, &Operator::Number { number: 4., x: 0, i: 1}), Some(Operator::Number { number: -1., x: 0, i: 1}));
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 3., x: 0, i: 1}, &Operator::Number { number: 4., x: 0, i: 2}), None);
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 3., x: 0, i: 0}, &Operator::Number { number: 4., x: 0, i: 2}), Some(Operator::Number { number: 7., x: 0, i: 0}));
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 3., x: 0, i: 0}, &Operator::Number { number: 4., x: 0, i: 98}), Some(Operator::Number { number: 7., x: 0, i: 0}));
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 3., x: 0, i: 1}, &Operator::Number { number: 4., x: 0, i: 3}), Some(Operator::Number { number: 7., x: 0, i: 1}));
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 0., x: 0, i: 1}, &Operator::Number { number: 4., x: 0, i: 3}), Some(Operator::Number { number: 4., x: 0, i: 1}));
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 0., x: 0, i: 0}, &Operator::Number { number: 4., x: 0, i: 3}), Some(Operator::Number { number: 4., x: 0, i: 1}));
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 4., x: 0, i: 3}, &Operator::Number { number: 0., x: 0, i: 3}), Some(Operator::Number { number: -4., x: 0, i: 1}));
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 4., x: 0, i: 3}, &Operator::Number { number: 0., x: 0, i: 0}), Some(Operator::Number { number: -4., x: 0, i: 1}));
    }

    #[test]
    fn sub_i_x() {
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 3., x: 1, i: 1}, &Operator::Number { number: 4., x: 1, i: 3}), Some(Operator::Number { number: 7., x: 1, i: 1}));
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 3., x: 1, i: 1}, &Operator::Number { number: 4., x: 0, i: 3}), None);
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 3., x: 2, i: 1}, &Operator::Number { number: 4., x: 2, i: 3}), Some(Operator::Number { number: 7., x: 2, i: 1}));
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 3., x: 2, i: 2}, &Operator::Number { number: 4., x: 2, i: 0}), Some(Operator::Number { number: -7., x: 2, i: 0}));
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 3., x: 2, i: 4}, &Operator::Number { number: 4., x: 2, i: 0}), Some(Operator::Number { number: -1., x: 2, i: 0}));
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 3., x: 2, i: 4}, &Operator::Number { number: 4., x: 2, i: 4}), Some(Operator::Number { number: -1., x: 2, i: 0}));
    }

    #[test]
    fn sub_nb_mat() {
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 3., x: 0, i: 2}, &Operator::Mat(vec![
            vec![Operator::Number { number: 1., x: 0, i: 0}, Operator::Number { number: 2., x: 0, i: 0}],
            vec![Operator::Number { number: 3., x: 0, i: 0}, Operator::Number { number: 4., x: 0, i: 0}]
        ])), Some(Operator::Mat(vec![
            vec![Operator::Number { number: -4., x: 0, i: 0}, Operator::Number { number: -5., x: 0, i: 0}],
            vec![Operator::Number { number: -6., x: 0, i: 0}, Operator::Number { number: -7., x: 0, i: 0}]
        ])));
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 3., x: 0, i: 3}, &Operator::Mat(vec![
            vec![Operator::Number { number: 1., x: 0, i: 0}, Operator::Number { number: 2., x: 0, i: 0}],
            vec![Operator::Number { number: 3., x: 0, i: 0}, Operator::Number { number: 4., x: 0, i: 0}]
        ])), None);
        assert_eq!(Operator::Minus.calc(&Operator::Number { number: 3., x: 1, i: 0}, &Operator::Mat(vec![
            vec![Operator::Number { number: 1., x: 0, i: 0}, Operator::Number { number: 2., x: 0, i: 0}],
            vec![Operator::Number { number: 3., x: 0, i: 0}, Operator::Number { number: 4., x: 0, i: 0}]
        ])), None);
        assert_eq!(Operator::Minus.calc(&Operator::Mat(vec![
            vec![Operator::Number { number: 1., x: 0, i: 0}, Operator::Number { number: 2., x: 0, i: 0}],
            vec![Operator::Number { number: 3., x: 0, i: 0}, Operator::Number { number: 4., x: 0, i: 0}]
        ]), &Operator::Number { number: 3., x: 0, i: 0}), Some(Operator::Mat(vec![
            vec![Operator::Number { number: -2., x: 0, i: 0}, Operator::Number { number: -1., x: 0, i: 0}],
            vec![Operator::Number { number: 0., x: 0, i: 0}, Operator::Number { number: 1., x: 0, i: 0}]
        ])));
        assert_eq!(Operator::Minus.calc(&Operator::Mat(vec![
            vec![Operator::Number { number: 1., x: 0, i: 0}, Operator::Number { number: -42., x: 0, i: 0}, Operator::Number { number: 3., x: 0, i: 0}],
            vec![Operator::Number { number: 4., x: 0, i: 0}, Operator::Number { number: 13., x: 0, i: 0}, Operator::Number { number: 6., x: 0, i: 0}],
            vec![Operator::Number { number: 0., x: 0, i: 0}, Operator::Number { number: 8., x: 0, i: 0}, Operator::Number { number: 9., x: 0, i: 0}]
        ]), &Operator::Number { number: 42., x: 0, i: 0}), Some(Operator::Mat(vec![
            vec![Operator::Number { number: -41., x: 0, i: 0}, Operator::Number { number: -84., x: 0, i: 0}, Operator::Number { number: -39., x: 0, i: 0}],
            vec![Operator::Number { number: -38., x: 0, i: 0}, Operator::Number { number: -29., x: 0, i: 0}, Operator::Number { number: -36., x: 0, i: 0}],
            vec![Operator::Number { number: -42., x: 0, i: 0}, Operator::Number { number: -34., x: 0, i: 0}, Operator::Number { number: -33., x: 0, i: 0}]
        ])));

    }

    #[test]
    fn sub_mat() {
        assert_eq!(Operator::Minus.calc(&Operator::Mat(vec![
            vec![Operator::Number { number: 99., x: 0, i: 0}, Operator::Number { number: 98., x: 0, i: 0}],
            vec![Operator::Number { number: 97., x: 0, i: 0}, Operator::Number { number: 96., x: 0, i: 0}]
        ]), &Operator::Mat(vec![
            vec![Operator::Number { number: 1., x: 0, i: 0}, Operator::Number { number: 2., x: 0, i: 0}],
            vec![Operator::Number { number: 3., x: 0, i: 0}, Operator::Number { number: 4., x: 0, i: 0}]
        ])), Some(Operator::Mat(vec![
            vec![Operator::Number { number: 98., x: 0, i: 0}, Operator::Number { number: 96., x: 0, i: 0}],
            vec![Operator::Number { number: 94., x: 0, i: 0}, Operator::Number { number: 92., x: 0, i: 0}]
        ])));
        assert_eq!(Operator::Minus.calc(&Operator::Mat(vec![
            vec![Operator::Number { number: 1., x: 0, i: 0}, Operator::Number { number: 213., x: 0, i: 0}, Operator::Number { number: 0., x: 0, i: 0}],
            vec![Operator::Number { number: -9., x: 0, i: 0}, Operator::Number { number: -92., x: 0, i: 0}, Operator::Number { number: -24., x: 0, i: 0}]
        ]), &Operator::Mat(vec![
            vec![Operator::Number { number: 123., x: 0, i: 0}, Operator::Number { number: 22., x: 0, i: 0}, Operator::Number { number: -22., x: 0, i: 0}],
            vec![Operator::Number { number: 982., x: 0, i: 0}, Operator::Number { number: 41., x: 0, i: 0}, Operator::Number { number: -42., x: 0, i: 0}]
        ])), Some(Operator::Mat(vec![
            vec![Operator::Number { number: -122., x: 0, i: 0}, Operator::Number { number: 191., x: 0, i: 0}, Operator::Number { number: 22., x: 0, i: 0}],
            vec![Operator::Number { number: -991., x: 0, i: 0}, Operator::Number { number: -133., x: 0, i: 0}, Operator::Number { number: 18., x: 0, i: 0}]
        ])));
        assert_eq!(Operator::Minus.calc(&Operator::Mat(vec![
            vec![Operator::Number { number: 1., x: 0, i: 0}, Operator::Number { number: 213., x: 0, i: 0}, Operator::Number { number: 0., x: 0, i: 0}],
            vec![Operator::Number { number: -9., x: 0, i: 0}, Operator::Number { number: -92., x: 0, i: 0}, Operator::Number { number: -24., x: 0, i: 0}]
        ]), &Operator::Mat(vec![
            vec![Operator::Number { number: 22., x: 0, i: 0}, Operator::Number { number: -22., x: 0, i: 0}],
            vec![Operator::Number { number: 982., x: 0, i: 0}, Operator::Number { number: 41., x: 0, i: 0}, Operator::Number { number: -42., x: 0, i: 0}]
        ])), None);
    }

}

mod power {

}

mod div {

}

mod mat_mult {

}