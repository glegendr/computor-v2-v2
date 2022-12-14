
use std::{fmt, collections::HashMap};

use crate::operator::Operator;
/* ---------- BTREE ---------- */
#[derive(Debug, Clone, PartialEq)]
pub struct BTree {
    pub c1: Option<Box<BTree>>,
    pub c2: Option<Box<BTree>>,
    pub node: Operator
}

impl fmt::Display for BTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{}", self.to_string()))
    }
}

impl BTree {
    pub fn new(node: Operator) -> BTree {
        BTree {
            c1: None,
            c2: None,
            node,
        }
    }

    fn find_depth_value(&self, depth: usize) -> Vec<Option<Operator>> {
        let mut ret = Vec::new();
        if depth == 0 {
            return vec![Some(self.node.clone())]
        }
        match &self.c1 {
            Some(c1) => ret.append(&mut c1.find_depth_value(depth - 1)),
            None => ret.append(&mut vec![None; 2_usize.pow((depth - 1) as u32)])
        }
        match &self.c2 {
            Some(c2) => ret.append(&mut c2.find_depth_value(depth - 1)),
            None => ret.append(&mut vec![None; 2_usize.pow((depth - 1) as u32)])
        }
        ret

    }

    #[allow(dead_code)]
    pub fn print(&self) {
        let mut staged_tree = Vec::new();
        let mut depth = 0;
        loop {
            let vec = self.find_depth_value(depth);
            if vec.iter().all(|x| x.is_none()) {
                break
            }
            staged_tree.push(vec);
            depth += 1;
        }
        let len = staged_tree.len();
        if len == 0 {
            return
        }
        let new_tree = staged_tree
            .iter()
            .map(|stage| stage
                .iter()
                .map(|ope| match ope {
                    Some(ope) => ope.to_string(),
                    None => String::from("")
                })
                .collect::<Vec<String>>()
            ).collect::<Vec<Vec<String>>>();
        let max_ope_len = new_tree.iter().fold(0, |acc, stage| acc.max(stage.iter().fold(0, |acc, ope| acc.max(ope.len()))));
        for (depth, stage) in new_tree.iter().enumerate() {
            let mut ret = String::new();
            let mut padding = max_ope_len;
            for _ in 0..len - depth - 1 {
                padding = 2 * padding + 1;
            }
            for ope in stage {
                ret = format!("{ret}{:^padding$}|", ope);
            }
            println!("|{ret}");
        }
    }

    pub fn insert_a(&mut self, sub_tree: BTree) {
        self.c1 = Some(Box::new(sub_tree));
    }

    pub fn insert_b(&mut self, sub_tree: BTree) {
        self.c2 = Some(Box::new(sub_tree));
    }

    pub fn to_string(&self) -> String {
        let mut ret = String::new();
        if let Some(c1) = &self.c1 {
            ret = format!("{c1}");
        }
        if let Some(c2) = &self.c2 {
            ret = format!("{ret}{c2}");
        }
        format!("{ret} {}", self.node)
    }

    pub fn to_vec(&self) -> Vec<Operator> {
        let mut ret = Vec::new();
        if let Some(c1) = &self.c1 {
            ret.append(&mut c1.to_vec());
        }
        if let Some(c2) = &self.c2 {
            ret.append(&mut c2.to_vec());
        }
        ret.push(self.node.clone());
        ret
    }

    pub fn from_vec(formula: &Vec<Operator>) -> Result<BTree, String> {
        let mut formula_clone = formula.clone();
        let tree = Self::from_vec_recursiv(&mut formula_clone)?;
        if !formula_clone.is_empty() {
            return Err(String::from("Wrong input given"));
        }
        tree.check_integrity()?;
        Ok(tree)
    }

    fn check_integrity(&self) -> Result<(), String> {
        match (&self.node, &self.c1, &self.c2) {
            (Operator::Var(_), _, _) | (Operator::Number { .. }, _, _) | (Operator::Mat(_), _, _) => {
                if self.c1.is_some() || self.c2.is_some() {
                    return Err(String::from("Children found for Variable, number or matrice"))
                }
            }
            (Operator::OpenParenthesis, _, _) | (Operator::CloseParenthesis, _, _) | (Operator::Equal, _, _) => return Err(String::from("Unexpected parentesis or equal")),
            (_, None, _) | (_, _, None) => return Err(String::from("Operator must have 2 children")),
            _ => {}
        }
        if let Some(c1) = &self.c1 {
            c1.check_integrity()?;
        }
        if let Some(c2) = &self.c2 {
            c2.check_integrity()?;
        }
        Ok(())
    }

    fn from_vec_recursiv(formula: &mut Vec<Operator>) -> Result<BTree, String> {
        if let Some(last_op) = formula.pop() {
            let mut ret = match last_op {
                Operator::Var(_) | Operator::Number { .. } | Operator::Mat(_) => return Ok(BTree::new(last_op)),
                Operator::OpenParenthesis | Operator::CloseParenthesis | Operator::Equal => Err("unexpected operator {last_op} in btree")?,
                op => BTree::new(op)
            };
            ret.insert_b(BTree::from_vec_recursiv(formula)?);
            ret.insert_a(BTree::from_vec_recursiv(formula)?);
            return Ok(ret)
        }
        Err(String::from("Error while parsing formula"))
    }


    pub fn eval(&self) -> Result<BTree, String> {
        if self.c1.is_none() && self.c2.is_none() {
            return Ok(self.clone())
        }
        let mut new_tree = self.clone();
        new_tree = new_tree.delete_minus();
        if let Some(c1) = &new_tree.c1 {
            new_tree.c1 = Some(Box::new(c1.eval()?));
        }
        if let Some(c2) = &new_tree.c2 {
            new_tree.c2 = Some(Box::new(c2.eval()?));
        }
        match (&new_tree.c1, &new_tree.c2) {
            (Some(c1), Some(c2)) => {
                if let Some(res) = new_tree.node.calc(&c1.node, &c2.node) {
                    new_tree.node = res;
                    new_tree.c1 = None;
                    new_tree.c2 = None;
                }
            },
            (None, None) => {
                match &new_tree.node {
                    Operator::Var(_) | Operator::Number { .. } | Operator::Mat(_) => {},
                    ope => return Err(format!("No child found for {ope}"))
                }
            },
            _ => return Err(String::from("Only one child founded"))
        }
        if new_tree.has_equivalent_precedence() {
            let clone = new_tree.clone();
            new_tree = new_tree.calc_equivalent();
            if clone != new_tree {
                new_tree = new_tree.eval()?;
            }
        }
        if new_tree.expand_power() {
            new_tree = new_tree.eval()?;
        }
        if new_tree.calc_mult() {
            new_tree = new_tree.eval()?;
        }
        Ok(new_tree)
    }

    fn expand_power(&mut self) -> bool {
        let mut ret = false;
        if let Operator::Power = self.node {
            if let (Some(c1), Some(c2)) = (&self.c1, &self.c2) {
                if let Operator::Number { number, x, i } = c2.node {
                    if x == 0 && i == 0 && number > 0. && number == number.round() && number < 5. {
                        ret = true;
                        let mut new_tree = *c1.clone();
                        for _ in 1..(number as i64) {
                            let mut new_tree_2 = BTree::new(Operator::Mult);
                            new_tree_2.insert_a(new_tree.clone());
                            new_tree_2.insert_b(*c1.clone());
                            new_tree = new_tree_2;
                        }
                        *self = new_tree;
                    }
                }
            }
        }
        ret
    }

    fn calc_equivalent(&self) -> Self {
        let mut ret = BTree::new(self.node.clone());
        let mut other = Vec::new();
        let vals = self.get_all_vals();
        let mut map: HashMap<(i32, i32), Vec<Operator>> = HashMap::new();
        for ope in vals {
            match ope {
                Operator::Number { x, i, .. } => {
                    match map.get_mut(&(x, i % 2)) {
                        Some(v) => v.push(ope),
                        None => {
                            map.insert((x, i % 2), vec![ope]);
                        }
                    }
                }
                Operator::Mat(_) => {
                    match map.get_mut(&(0, 0)) {
                        Some(v) => v.push(ope),
                        None => {
                            map.insert((0, 0), vec![ope]);
                        }
                    }
                }
                Operator::Var(_) => other.push(ope),
                _ => continue
            }
        }
        for ((x, i), v) in map {
            let init = match &self.node {
                Operator::Add => Operator::Number { number: 0., x, i },
                Operator::Mult => Operator::Number { number: 1., x: 0, i: 0 },
                _ => return self.clone()
            };
            match v.iter().fold(Some(init), |acc: Option<Operator>, ope| match acc {
                Some(acc2) => return self.node.calc(&acc2, ope),
                None => return None
            }) {
                Some(res) => {
                    match (&ret.c1, &ret.c2) {
                        (None, _) => ret.insert_a(BTree::new(res)),
                        (_, None) => ret.insert_b(BTree::new(res)),
                        _ => {
                            let mut new_ret = BTree::new(self.node.clone());
                            new_ret.insert_a(ret);
                            new_ret.insert_b(BTree::new(res));
                            ret = new_ret;
                        }
                    }
                },
                None => return self.clone(),
            }
        }
        for vars in other {
            match (&ret.c1, &ret.c2) {
                (None, _) => ret.insert_a(BTree::new(vars)),
                (_, None) => ret.insert_b(BTree::new(vars)),
                _ => {
                    let mut new_ret = BTree::new(self.node.clone());
                    new_ret.insert_a(ret);
                    new_ret.insert_b(BTree::new(vars));
                    ret = new_ret;
                }
            }

        }
        match (&ret.c1, &ret.c2) {
            (Some(_), Some(_))  | (None, None)=> {},
            (Some(c1), None) => ret = *c1.clone(),
            (None, Some(c2)) => ret = *c2.clone(),
        }
        ret
    }

    pub fn get_all_vals(&self) -> Vec<Operator> {
        match &self.node {
            Operator::Number { .. } | Operator::Mat(_) | Operator::Var(_) => vec![self.node.clone()],
            _ => {
                let mut ret = Vec::new();
                if let Some(c1) = &self.c1 {
                    ret.append(&mut c1.get_all_vals());
                }
                if let Some(c2) = &self.c2 {
                    ret.append(&mut c2.get_all_vals());
                }
                ret
            }
        }
    }

    pub fn all(&self, f: fn(&Self) -> bool) -> bool {
        let mut ret = f(self);
        if let Some(c1) = &self.c1 {
            ret = ret && c1.all(f);
        }
        if let Some(c2) = &self.c2 {
            ret = ret && c2.all(f);
        }
        ret
    }

    fn calc_mult(&mut self) -> bool {
        if let Operator::Mult = &self.node {
            match (&self.c1, &self.c2) {
                (Some(c1), Some(c2)) => {
                    if c1.all(|tree| match &tree.node {
                        Operator::Add | Operator::Number { .. } | Operator::Var(_) | Operator::Mat(_) => true,
                        _ => false
                    }) && c2.all(|tree| match &tree.node {
                        Operator::Add | Operator::Number { .. } | Operator::Var(_) | Operator::Mat(_) => true,
                        _ => false
                    }) {
                        let c1_vals = c1.get_all_vals();
                        let c2_vals = c2.get_all_vals();
                        let mut ret = BTree::new(Operator::Add);
                        for c1_ope in &c1_vals {
                            for c2_ope in &c2_vals {
                                match Operator::Mult.calc(c1_ope, c2_ope) {
                                    Some(res) => {
                                        match (&ret.c1, &ret.c2) {
                                            (None, _) => ret.insert_a(BTree::new(res)),
                                            (_, None) => ret.insert_b(BTree::new(res)),
                                            _ => {
                                                let mut new_ret = BTree::new(Operator::Add);
                                                new_ret.insert_a(ret);
                                                new_ret.insert_b(BTree::new(res));
                                                ret = new_ret;
                                            }
                                        }
                                    }
                                    _ => return false
                                }
                            }
                        }
                        match (&ret.c1, &ret.c2) {
                            (Some(c1), None) => *self = *c1.clone(),
                            (None, Some(c2)) => *self = *c2.clone(),
                            (None, None) => return false,
                            _ => *self = ret
                        }
                        return true
                    }
                },
                _ => {}
            }
        }
        false
    }

    fn has_equivalent_precedence(&self) -> bool {
        match &self.node {
            Operator::Var(_) | Operator::Number { .. } | Operator::Mat(_) => true,
            Operator::Minus | Operator::Div | Operator::Power | Operator::Modulo => false,
            ope => {
                let mut ret = true;
                let precedence = ope.get_precedence();
                if let Some(c1) = &self.c1 {
                    ret = ret && c1.has_equivalent_precedence_rec(precedence);
                }
                if let Some(c2) = &self.c2 {
                    ret = ret && c2.has_equivalent_precedence_rec(precedence);
                }
                ret
            }
        }
    }

    fn has_equivalent_precedence_rec(&self, precedence: u8) -> bool {
        match &self.node {
            Operator::Var(_) | Operator::Number { .. } | Operator::Mat(_) => true,
            Operator::Minus | Operator::Div | Operator::Power | Operator::Modulo => false,
            ope => {
                let mut ret = ope.get_precedence() == precedence;
                if let Some(c1) = &self.c1 {
                    ret = ret && c1.has_equivalent_precedence_rec(precedence);
                }
                if let Some(c2) = &self.c2 {
                    ret = ret && c2.has_equivalent_precedence_rec(precedence);
                }
                ret
            }
        }
    }

    fn delete_minus(&self) -> BTree {
        match &self.node {
            Operator::Minus => {
                let mut ret = BTree::new(Operator::Add);
                if let Some(c1) = &self.c1 {
                    ret.insert_a(c1.delete_minus());
                }
                if let Some(c2) = &self.c2 {
                    let mut new_c2 = BTree::new(Operator::Mult);
                    new_c2.insert_a(BTree::new(Operator::Number { number: -1., x: 0, i: 0 }));
                    new_c2.insert_b(c2.delete_minus());
                    ret.insert_b(new_c2);
                }
                ret
            },
            ope => {
                let mut ret = BTree::new(ope.clone());
                if let Some(c1) = &self.c1 {
                    ret.insert_a(c1.delete_minus());
                }
                if let Some(c2) = &self.c2 {
                    ret.insert_b(c2.delete_minus());
                }
                ret
            }
        }
    }

    pub fn change_x(&self, by: f64) -> Self {
        let mut ret = self.clone();
        if let Operator::Number { number, x, i } = ret.node {
            ret.node = Operator::Number { number: number * by.powf(x as f64), x: 0, i };
        }
        if let Some(c1) = ret.c1 {
           ret.c1 = Some(Box::new(c1.change_x(by)));
        }
        if let Some(c2) = ret.c2 {
           ret.c2 = Some(Box::new(c2.change_x(by)));
        }
        ret
    }
}
