use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use crate::ast::*;
use crate::parser::Parser;

use log::info;
use tracing::instrument;

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Symbol(String),
    Boolean(bool),
    Nil,
    Array(Rc<RefCell<Vec<Value>>>),
    Hash(Rc<RefCell<HashMap<String, Value>>>),
    Lambda(Vec<String>, Vec<Statement>, Rc<RefCell<Environment>>),
    Method(MethodDef),
    System(SystemDef),
    Message(MessageDef),
    Instance(String, Rc<RefCell<HashMap<String, Value>>>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }
}

impl Value {
    #[instrument(skip(self))]
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Nil | Value::Boolean(false) => false,
            _ => true,
        }
    }

    #[instrument(skip(self))]
    pub fn to_string(&self) -> String {
        match self {
            Value::Integer(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::String(s) => s.clone(),
            Value::Symbol(s) => format!(":{}", s),
            Value::Boolean(b) => b.to_string(),
            Value::Nil => "nil".to_string(),
            Value::Array(arr) => {
                let arr = arr.borrow();
                let elements: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                format!("[{}]", elements.join(", "))
            }
            Value::Hash(hash) => {
                let hash = hash.borrow();
                let pairs: Vec<String> = hash
                    .iter()
                    .map(|(k, v)| format!("{} => {}", k, v.to_string()))
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
            Value::Lambda(_, _, _) => "#<Lambda>".to_string(),
            Value::Method(m) => format!("#<Method: {}>", m.name),
            Value::System(s) => format!("#<System: {}>", s.name),
            Value::Message(m) => format!("#<Message: {}>", m.name),
            Value::Instance(name, _) => format!("#<Instance of {}>", name),
        }
    }

    #[instrument(skip(self))]
    pub fn to_integer(&self) -> Result<i64, String> {
        match self {
            Value::Integer(n) => Ok(*n),
            Value::Float(f) => Ok(*f as i64),
            Value::String(s) => s
                .parse()
                .map_err(|_| format!("Cannot convert '{}' to integer", s)),
            Value::Boolean(true) => Ok(1),
            Value::Boolean(false) => Ok(0),
            _ => Err(format!("Cannot convert {:?} to integer", self)),
        }
    }

    #[instrument(skip(self))]
    pub fn to_float(&self) -> Result<f64, String> {
        match self {
            Value::Integer(n) => Ok(*n as f64),
            Value::Float(f) => Ok(*f),
            Value::String(s) => s
                .parse()
                .map_err(|_| format!("Cannot convert '{}' to float", s)),
            _ => Err(format!("Cannot convert {:?} to float", self)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            scopes: vec![HashMap::new()],
        }
    }

    #[instrument(skip(self))]
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    #[instrument(skip(self))]
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    #[instrument(skip(self))]
    pub fn define(&mut self, name: String, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }

    #[instrument(skip(self))]
    pub fn get(&self, name: &str) -> Option<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value.clone());
            }
        }
        None
    }

    #[instrument(skip(self))]
    pub fn set(&mut self, name: &str, value: Value) -> Result<(), String> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return Ok(());
            }
        }
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), value);
            Ok(())
        } else {
            Err(format!("No scope available to set variable '{}'", name))
        }
    }
}

#[derive(Clone)]
pub enum FlowControl {
    None,
    Return(Value),
    Break,
    Next,
}

pub struct Interpreter {
    debug: bool,
    env: Rc<RefCell<Environment>>,
    systems: HashMap<String, SystemDef>,
    messages: HashMap<String, MessageDef>,
    modules: HashMap<String, ModuleDef>,
    search_paths: Vec<PathBuf>,
    output: Vec<String>,
    current_file: Option<PathBuf>,
    loaded_files: Vec<String>,
}

impl Interpreter {
    pub fn new(path: Option<PathBuf>, search_paths: Vec<PathBuf>) -> Self {
        let mut interpreter = Interpreter {
            debug: false,
            env: Rc::new(RefCell::new(Environment::new())),
            systems: HashMap::new(),
            messages: HashMap::new(),
            modules: HashMap::new(),
            output: Vec::new(),
            current_file: path,
            search_paths: search_paths,
            loaded_files: Vec::new(),
        };

        interpreter.register_builtins();
        interpreter
    }

    #[instrument(skip(self))]
    fn register_builtins(&mut self) {
        let println_method = MethodDef {
            name: "println".to_string(),
            params: vec![],
            body: vec![],
            attributes: vec![],
        };
        self.env
            .borrow_mut()
            .define("println".to_string(), Value::Method(println_method));
    }

    #[instrument(skip(self))]
    pub fn get_output(&self) -> Vec<String> {
        self.output.clone()
    }

    pub fn run(&mut self, program: &Program) -> Result<Value, String> {
        match program {
            Program::Statements(statements) => {
                let mut result = Value::Nil;
                for statement in statements {
                    match statement {
                        Statement::Expression(expr) => {
                            result = self.evaluate_expression(expr)?;
                        }
                        _ => match self.execute_statement(statement)? {
                            FlowControl::Return(val) => return Ok(val),
                            FlowControl::Break => return Err("Break outside of loop".to_string()),
                            FlowControl::Next => return Err("Next outside of loop".to_string()),
                            FlowControl::None => {}
                        },
                    }
                }
                Ok(result)
            }
        }
    }

    #[instrument(skip(self))]
    fn execute_statement(&mut self, statement: &Statement) -> Result<FlowControl, String> {
        match statement {
            Statement::Expression(expr) => {
                self.evaluate_expression(expr)?;
                Ok(FlowControl::None)
            }
            Statement::Assignment(name, expr) => {
                let value = self.evaluate_expression(expr)?;
                self.env.borrow_mut().set(name, value)?;
                Ok(FlowControl::None)
            }
            Statement::MethodDef(method_def) => {
                let value = Value::Method(method_def.clone());
                self.env.borrow_mut().define(method_def.name.clone(), value);
                Ok(FlowControl::None)
            }
            Statement::SystemDef(system_def) => {
                self.systems
                    .insert(system_def.name.clone(), system_def.clone());
                let value = Value::System(system_def.clone());
                self.env.borrow_mut().define(system_def.name.clone(), value);
                Ok(FlowControl::None)
            }
            Statement::MessageDef(message_def) => {
                self.messages
                    .insert(message_def.name.clone(), message_def.clone());
                let value = Value::Message(message_def.clone());
                self.env
                    .borrow_mut()
                    .define(message_def.name.clone(), value);
                Ok(FlowControl::None)
            }
            Statement::If(if_stmt) => self.execute_if(if_stmt),
            Statement::While(while_stmt) => self.execute_while(while_stmt),
            Statement::For(for_stmt) => self.execute_for(for_stmt),
            Statement::Return(expr) => {
                let value = if let Some(e) = expr {
                    self.evaluate_expression(e)?
                } else {
                    Value::Nil
                };
                Ok(FlowControl::Return(value))
            }
            Statement::Break => Ok(FlowControl::Break),
            Statement::Next => Ok(FlowControl::Next),
            Statement::Attributes(_) => Ok(FlowControl::None),
            Statement::ModuleDef(module_def) => {
                self.modules
                    .insert(module_def.name.clone(), module_def.clone());
                for stmt in &module_def.body {
                    self.execute_statement(stmt)?;
                }
                Ok(FlowControl::None)
            }
            Statement::Require(path) => {
                self.require_file(path)?;
                Ok(FlowControl::None)
            }
        }
    }

    #[instrument(skip(self))]
    fn execute_if(&mut self, if_stmt: &IfStatement) -> Result<FlowControl, String> {
        let condition = self.evaluate_expression(&if_stmt.condition)?;

        if condition.is_truthy() {
            return self.execute_block(&if_stmt.then_block);
        }

        for (elsif_cond, elsif_block) in &if_stmt.elsif_blocks {
            let cond_val = self.evaluate_expression(elsif_cond)?;
            if cond_val.is_truthy() {
                return self.execute_block(elsif_block);
            }
        }

        if let Some(else_block) = &if_stmt.else_block {
            return self.execute_block(else_block);
        }

        Ok(FlowControl::None)
    }

    #[instrument(skip(self))]
    fn execute_while(&mut self, while_stmt: &WhileStatement) -> Result<FlowControl, String> {
        loop {
            let condition = self.evaluate_expression(&while_stmt.condition)?;
            if !condition.is_truthy() {
                break;
            }

            match self.execute_block(&while_stmt.body)? {
                FlowControl::Break => break,
                FlowControl::Next => continue,
                FlowControl::Return(val) => return Ok(FlowControl::Return(val)),
                FlowControl::None => {}
            }
        }
        Ok(FlowControl::None)
    }

    #[instrument(skip(self))]
    fn execute_for(&mut self, for_stmt: &ForStatement) -> Result<FlowControl, String> {
        let iterable = self.evaluate_expression(&for_stmt.iterable)?;

        let items = match iterable {
            Value::Array(arr) => arr.borrow().clone(),
            Value::Integer(start) => {
                vec![Value::Integer(start)]
            }
            _ => return Err(format!("Cannot iterate over {:?}", iterable)),
        };

        self.env.borrow_mut().push_scope();

        for item in items {
            self.env.borrow_mut().set(&for_stmt.variable, item)?;

            match self.execute_block(&for_stmt.body)? {
                FlowControl::Break => break,
                FlowControl::Next => continue,
                FlowControl::Return(val) => {
                    self.env.borrow_mut().pop_scope();
                    return Ok(FlowControl::Return(val));
                }
                FlowControl::None => {}
            }
        }

        self.env.borrow_mut().pop_scope();
        Ok(FlowControl::None)
    }

    #[instrument(skip(self))]
    fn execute_block(&mut self, statements: &[Statement]) -> Result<FlowControl, String> {
        if self.debug {
            info!("Executing block")
        }
        for statement in statements {
            match self.execute_statement(statement)? {
                FlowControl::None => {}
                flow => return Ok(flow),
            }
        }
        Ok(FlowControl::None)
    }

    #[instrument(skip(self))]
    fn evaluate_expression(&mut self, expr: &Expression) -> Result<Value, String> {
        match expr {
            Expression::Integer(n) => Ok(Value::Integer(*n)),
            Expression::Float(f) => Ok(Value::Float(*f)),
            Expression::String(s) => Ok(Value::String(s.clone())),
            Expression::StringInterpolation(parts) => self.evaluate_string_interpolation(parts),
            Expression::Symbol(s) => Ok(Value::Symbol(s.clone())),
            Expression::Boolean(b) => Ok(Value::Boolean(*b)),
            Expression::Nil => Ok(Value::Nil),
            Expression::Identifier(name) => {
                if name == "self" {
                    return Ok(Value::Nil);
                }
                self.env
                    .borrow()
                    .get(name)
                    .ok_or_else(|| format!("Undefined variable '{}'", name))
            }
            Expression::Array(elements) => {
                let mut values = Vec::new();
                for elem in elements {
                    values.push(self.evaluate_expression(elem)?);
                }
                Ok(Value::Array(Rc::new(RefCell::new(values))))
            }
            Expression::Hash(pairs) => {
                let mut hash = HashMap::new();
                for (key_expr, value_expr) in pairs {
                    let key = match self.evaluate_expression(key_expr)? {
                        Value::Symbol(s) => s,
                        Value::String(s) => s,
                        v => v.to_string(),
                    };
                    let value = self.evaluate_expression(value_expr)?;
                    hash.insert(key, value);
                }
                Ok(Value::Hash(Rc::new(RefCell::new(hash))))
            }
            Expression::Range {
                start,
                end,
                exclusive,
            } => {
                let start_val = self.evaluate_expression(start)?;
                let end_val = self.evaluate_expression(end)?;

                let start_int = start_val.to_integer()?;
                let end_int = end_val.to_integer()?;

                let end_adjusted = if *exclusive { end_int - 1 } else { end_int };

                let mut values = Vec::new();
                for i in start_int..=end_adjusted {
                    values.push(Value::Integer(i));
                }
                Ok(Value::Array(Rc::new(RefCell::new(values))))
            }
            Expression::Binary(op, left, right) => self.evaluate_binary_op(*op, left, right),
            Expression::Unary(op, expr) => self.evaluate_unary_op(*op, expr),
            Expression::Call {
                receiver,
                method,
                args,
            } => self.evaluate_call(receiver, method, args),
            Expression::Index { expr, index } => self.evaluate_index(expr, index),
            Expression::Lambda { params, body } => Ok(Value::Lambda(
                params.clone(),
                body.clone(),
                Rc::clone(&self.env),
            )),
            Expression::NamespaceAccess(parts) => self.resolve_namespace(parts),
            Expression::Block(statements) => {
                self.env.borrow_mut().push_scope();
                let result = Value::Nil;
                for stmt in statements {
                    match self.execute_statement(stmt)? {
                        FlowControl::Return(val) => {
                            self.env.borrow_mut().pop_scope();
                            return Ok(val);
                        }
                        FlowControl::Break | FlowControl::Next => {
                            self.env.borrow_mut().pop_scope();
                            return Err("Break/Next in block expression".to_string());
                        }
                        FlowControl::None => {}
                    }
                }
                self.env.borrow_mut().pop_scope();
                Ok(result)
            }
        }
    }

    #[instrument(skip(self))]
    fn evaluate_binary_op(
        &mut self,
        op: BinaryOp,
        left: &Expression,
        right: &Expression,
    ) -> Result<Value, String> {
        let left_val = self.evaluate_expression(left)?;

        match op {
            BinaryOp::And => {
                if !left_val.is_truthy() {
                    return Ok(left_val);
                }
                return self.evaluate_expression(right);
            }
            BinaryOp::Or => {
                if left_val.is_truthy() {
                    return Ok(left_val);
                }
                return self.evaluate_expression(right);
            }
            _ => {}
        }

        let right_val = self.evaluate_expression(right)?;

        match op {
            BinaryOp::Add => self.add_values(left_val, right_val),
            BinaryOp::Subtract => self.subtract_values(left_val, right_val),
            BinaryOp::Multiply => self.multiply_values(left_val, right_val),
            BinaryOp::Divide => self.divide_values(left_val, right_val),
            BinaryOp::Modulo => self.modulo_values(left_val, right_val),
            BinaryOp::Power => self.power_values(left_val, right_val),
            BinaryOp::Equal => Ok(Value::Boolean(self.values_equal(&left_val, &right_val))),
            BinaryOp::NotEqual => Ok(Value::Boolean(!self.values_equal(&left_val, &right_val))),
            BinaryOp::LessThan => self.compare_values(left_val, right_val, |a, b| a < b),
            BinaryOp::LessThanOrEqual => self.compare_values(left_val, right_val, |a, b| a <= b),
            BinaryOp::GreaterThan => self.compare_values(left_val, right_val, |a, b| a > b),
            BinaryOp::GreaterThanOrEqual => self.compare_values(left_val, right_val, |a, b| a >= b),
            BinaryOp::Spaceship => self.spaceship_values(left_val, right_val),
            BinaryOp::BitwiseAnd => self.bitwise_and(left_val, right_val),
            BinaryOp::BitwiseOr => self.bitwise_or(left_val, right_val),
            BinaryOp::BitwiseXor => self.bitwise_xor(left_val, right_val),
            BinaryOp::LeftShift => self.left_shift(left_val, right_val),
            BinaryOp::RightShift => self.right_shift(left_val, right_val),
            BinaryOp::And | BinaryOp::Or => unreachable!("Already handled above"),
        }
    }

    #[instrument(skip(self))]
    fn evaluate_unary_op(&mut self, op: UnaryOp, expr: &Expression) -> Result<Value, String> {
        let val = self.evaluate_expression(expr)?;
        match op {
            UnaryOp::Negate => match val {
                Value::Integer(n) => Ok(Value::Integer(-n)),
                Value::Float(f) => Ok(Value::Float(-f)),
                _ => Err(format!("Cannot negate {:?}", val)),
            },
            UnaryOp::Not => Ok(Value::Boolean(!val.is_truthy())),
            UnaryOp::BitwiseNot => match val {
                Value::Integer(n) => Ok(Value::Integer(!n)),
                _ => Err(format!("Cannot apply bitwise NOT to {:?}", val)),
            },
        }
    }

    #[instrument(skip(self))]
    fn evaluate_call(
        &mut self,
        receiver: &Expression,
        method: &str,
        args: &[Expression],
    ) -> Result<Value, String> {
        if let Expression::Identifier(name) = receiver {
            if name == "println" || name == "self" && method == "println" {
                for arg in args {
                    let val = self.evaluate_expression(arg)?;
                    let output = val.to_string();
                    self.output.push(output.clone());
                    println!("{}", output);
                }
                return Ok(Value::Nil);
            }

            if name == "self" {
                let method_opt = self.env.borrow().get(method);
                if let Some(Value::Method(method_def)) = method_opt {
                    return self.call_method(&method_def, args);
                }
                return Err(format!("Undefined function '{}'", method));
            }
        }

        if method == "println" {
            for arg in args {
                let val = self.evaluate_expression(arg)?;
                let output = val.to_string();
                self.output.push(output.clone());
                println!("{}", output);
            }
            return Ok(Value::Nil);
        }

        let receiver_val = self.evaluate_expression(receiver)?;

        match method {
            "length" | "size" => match receiver_val {
                Value::Array(arr) => Ok(Value::Integer(arr.borrow().len() as i64)),
                Value::String(s) => Ok(Value::Integer(s.len() as i64)),
                _ => Err(format!("{:?} does not have a length method", receiver_val)),
            },
            "push" => match receiver_val {
                Value::Array(arr) => {
                    for arg in args {
                        let val = self.evaluate_expression(arg)?;
                        arr.borrow_mut().push(val);
                    }
                    Ok(Value::Array(arr))
                }
                _ => Err(format!("{:?} does not have a push method", receiver_val)),
            },
            "new" => match receiver_val {
                Value::System(system_def) => {
                    let instance_fields = Rc::new(RefCell::new(HashMap::new()));
                    let instance =
                        Value::Instance(system_def.name.clone(), Rc::clone(&instance_fields));

                    for stmt in &system_def.body {
                        if let Statement::MethodDef(method_def) = stmt {
                            if method_def.name == "initialize" {
                                self.call_instance_method(&instance, method_def, args)?;
                                break;
                            }
                        }
                    }

                    Ok(instance)
                }
                _ => Err(format!("{:?} is not a system", receiver_val)),
            },
            _ => match receiver_val {
                Value::Instance(ref system_name, _) => {
                    if let Some(system_def) = self.systems.get(system_name).cloned() {
                        for stmt in &system_def.body {
                            if let Statement::MethodDef(method_def) = stmt {
                                if method_def.name == method {
                                    return self.call_instance_method(
                                        &receiver_val,
                                        method_def,
                                        args,
                                    );
                                }
                            }
                        }
                    }
                    Err(format!(
                        "Method '{}' not found on instance of {}",
                        method, system_name
                    ))
                }
                _ => {
                    let method_opt = self.env.borrow().get(method);
                    if let Some(Value::Method(method_def)) = method_opt {
                        self.call_method(&method_def, args)
                    } else {
                        Err(format!("Method '{}' not found", method))
                    }
                }
            },
        }
    }

    #[instrument(skip(self))]
    fn call_method(
        &mut self,
        method_def: &MethodDef,
        args: &[Expression],
    ) -> Result<Value, String> {
        if args.len() != method_def.params.len() {
            return Err(format!(
                "Method '{}' expects {} arguments but got {}",
                method_def.name,
                method_def.params.len(),
                args.len()
            ));
        }

        self.env.borrow_mut().push_scope();

        for (param, arg) in method_def.params.iter().zip(args) {
            let val = self.evaluate_expression(arg)?;
            self.env.borrow_mut().define(param.clone(), val);
        }

        let result = match self.execute_block(&method_def.body)? {
            FlowControl::Return(val) => val,
            FlowControl::None => Value::Nil,
            FlowControl::Break => return Err("Break outside of loop".to_string()),
            FlowControl::Next => return Err("Next outside of loop".to_string()),
        };

        self.env.borrow_mut().pop_scope();
        Ok(result)
    }

    #[instrument(skip(self))]
    fn call_instance_method(
        &mut self,
        instance: &Value,
        method_def: &MethodDef,
        args: &[Expression],
    ) -> Result<Value, String> {
        if args.len() != method_def.params.len() {
            return Err(format!(
                "Method '{}' expects {} arguments but got {}",
                method_def.name,
                method_def.params.len(),
                args.len()
            ));
        }

        if let Value::Instance(_, fields) = instance {
            self.env.borrow_mut().push_scope();

            for (name, value) in fields.borrow().iter() {
                self.env.borrow_mut().define(name.clone(), value.clone());
            }

            for (param, arg) in method_def.params.iter().zip(args) {
                let val = self.evaluate_expression(arg)?;
                self.env.borrow_mut().define(param.clone(), val);
            }

            let result = match self.execute_block(&method_def.body)? {
                FlowControl::Return(val) => val,
                FlowControl::None => Value::Nil,
                FlowControl::Break => return Err("Break outside of loop".to_string()),
                FlowControl::Next => return Err("Next outside of loop".to_string()),
            };

            let field_names: Vec<String> = fields.borrow().keys().cloned().collect();
            for name in field_names {
                if let Some(new_val) = self.env.borrow().get(&name) {
                    fields.borrow_mut().insert(name.clone(), new_val);
                }
            }

            let new_instance_vars: Vec<(String, Value)> = {
                let env_borrow = self.env.borrow();
                if let Some(scope) = env_borrow.scopes.last() {
                    scope
                        .iter()
                        .filter(|(name, _)| name.starts_with('@'))
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect()
                } else {
                    Vec::new()
                }
            };

            for (name, value) in new_instance_vars {
                fields.borrow_mut().insert(name, value);
            }

            self.env.borrow_mut().pop_scope();
            Ok(result)
        } else {
            Err(format!("Expected instance, got {:?}", instance))
        }
    }

    #[instrument(skip(self))]
    fn evaluate_string_interpolation(
        &mut self,
        parts: &[InterpolationPart],
    ) -> Result<Value, String> {
        let mut result = String::new();

        for part in parts {
            match part {
                InterpolationPart::Literal(s) => {
                    result.push_str(s);
                }
                InterpolationPart::Expression(expr) => {
                    let value = self.evaluate_expression(expr)?;
                    result.push_str(&value.to_string());
                }
            }
        }

        Ok(Value::String(result))
    }

    #[instrument(skip(self))]
    fn evaluate_index(&mut self, expr: &Expression, index: &Expression) -> Result<Value, String> {
        let array_val = self.evaluate_expression(expr)?;
        let index_val = self.evaluate_expression(index)?;

        match array_val {
            Value::Array(arr) => {
                let idx = index_val.to_integer()?;
                let arr_borrow = arr.borrow();
                if idx < 0 || idx >= arr_borrow.len() as i64 {
                    Ok(Value::Nil)
                } else {
                    Ok(arr_borrow[idx as usize].clone())
                }
            }
            Value::Hash(hash) => {
                let key = match index_val {
                    Value::Symbol(s) => s,
                    Value::String(s) => s,
                    v => v.to_string(),
                };
                Ok(hash.borrow().get(&key).cloned().unwrap_or(Value::Nil))
            }
            _ => Err(format!("Cannot index {:?}", array_val)),
        }
    }

    #[instrument(skip(self))]
    fn add_values(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(a as f64 + b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + b as f64)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            (a, b) => Err(format!("Cannot add {:?} and {:?}", a, b)),
        }
    }

    #[instrument(skip(self))]
    fn subtract_values(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(a as f64 - b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - b as f64)),
            (a, b) => Err(format!("Cannot subtract {:?} and {:?}", a, b)),
        }
    }

    #[instrument(skip(self))]
    fn multiply_values(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(a as f64 * b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a * b as f64)),
            (a, b) => Err(format!("Cannot multiply {:?} and {:?}", a, b)),
        }
    }

    #[instrument(skip(self))]
    fn divide_values(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if b == 0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::Integer(a / b))
                }
            }
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(a as f64 / b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a / b as f64)),
            (a, b) => Err(format!("Cannot divide {:?} and {:?}", a, b)),
        }
    }

    #[instrument(skip(self))]
    fn modulo_values(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if b == 0 {
                    Err("Modulo by zero".to_string())
                } else {
                    Ok(Value::Integer(a % b))
                }
            }
            (a, b) => Err(format!("Cannot modulo {:?} and {:?}", a, b)),
        }
    }

    #[instrument(skip(self))]
    fn power_values(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if b >= 0 {
                    Ok(Value::Integer(a.pow(b as u32)))
                } else {
                    Ok(Value::Float((a as f64).powf(b as f64)))
                }
            }
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(b))),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float((a as f64).powf(b))),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a.powf(b as f64))),
            (a, b) => Err(format!("Cannot raise {:?} to the power of {:?}", a, b)),
        }
    }

    #[instrument(skip(self))]
    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }

    fn compare_values<F>(&self, left: Value, right: Value, cmp: F) -> Result<Value, String>
    where
        F: Fn(f64, f64) -> bool,
    {
        let left_f = left.to_float()?;
        let right_f = right.to_float()?;
        Ok(Value::Boolean(cmp(left_f, right_f)))
    }

    #[instrument(skip(self))]
    fn spaceship_values(&self, left: Value, right: Value) -> Result<Value, String> {
        let left_f = left.to_float()?;
        let right_f = right.to_float()?;

        if left_f < right_f {
            Ok(Value::Integer(-1))
        } else if left_f > right_f {
            Ok(Value::Integer(1))
        } else {
            Ok(Value::Integer(0))
        }
    }

    #[instrument(skip(self))]
    fn bitwise_and(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a & b)),
            (a, b) => Err(format!("Cannot apply bitwise AND to {:?} and {:?}", a, b)),
        }
    }

    #[instrument(skip(self))]
    fn bitwise_or(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a | b)),
            (a, b) => Err(format!("Cannot apply bitwise OR to {:?} and {:?}", a, b)),
        }
    }

    #[instrument(skip(self))]
    fn bitwise_xor(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a ^ b)),
            (a, b) => Err(format!("Cannot apply bitwise XOR to {:?} and {:?}", a, b)),
        }
    }

    #[instrument(skip(self))]
    fn left_shift(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if b < 0 {
                    Err("Cannot left shift by negative amount".to_string())
                } else {
                    Ok(Value::Integer(a << b))
                }
            }
            (a, b) => Err(format!("Cannot apply left shift to {:?} and {:?}", a, b)),
        }
    }

    #[instrument(skip(self))]
    fn right_shift(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if b < 0 {
                    Err("Cannot right shift by negative amount".to_string())
                } else {
                    Ok(Value::Integer(a >> b))
                }
            }
            (a, b) => Err(format!("Cannot apply right shift to {:?} and {:?}", a, b)),
        }
    }

    #[instrument(skip(self))]
    fn resolve_path(&mut self, path: &str) -> Result<PathBuf, String> {
        let mut path = PathBuf::from(path);
        path.set_extension("rb");
        if std::fs::exists(&path).unwrap() {
            return Ok(PathBuf::from(path));
        }
        if let Some(file) = &self.current_file {
            let path = file.parent().unwrap().join(&path);
            if std::fs::exists(&path).unwrap() {
                return Ok(path);
            }
        }
        for p in &self.search_paths {
            let path = p.join(&path);
            if std::fs::exists(&path).unwrap() {
                return Ok(path);
            }
        }
        Err(format!("Could not find file: {}", path.display()))
    }

    #[instrument(skip(self))]
    fn require_file(&mut self, path: &str) -> Result<(), String> {
        let full_path = self.resolve_path(path)?;
        let full_path_str = full_path.to_string_lossy().to_string();

        if self.loaded_files.contains(&full_path_str) {
            return Ok(());
        }

        let file_content = fs::read_to_string(&full_path)
            .map_err(|e| format!("Failed to load file '{}': {}", full_path.display(), e))?;

        self.loaded_files.push(full_path_str.clone());

        let previous_file = self.current_file.clone();
        self.current_file = Some(PathBuf::from(full_path_str));

        let mut parser = Parser::new(&file_content);
        let program = parser
            .parse()
            .map_err(|e| format!("Parse error in '{}': {}", full_path.display(), e.message))?;

        match program {
            Program::Statements(statements) => {
                for statement in statements {
                    self.execute_statement(&statement)?;
                }
            }
        }

        self.current_file = previous_file;

        Ok(())
    }

    #[instrument(skip(self))]
    fn resolve_namespace(&self, parts: &[String]) -> Result<Value, String> {
        if parts.is_empty() {
            return Err("Empty namespace path".to_string());
        }

        let first = &parts[0];

        if self.modules.contains_key(first) {
            if parts.len() == 1 {
                return Ok(Value::Symbol(first.clone()));
            }

            let module_def = self.modules.get(first).unwrap();
            let target = &parts[parts.len() - 1];

            for stmt in &module_def.body {
                match stmt {
                    Statement::SystemDef(system_def) if &system_def.name == target => {
                        return Ok(Value::System(system_def.clone()));
                    }
                    Statement::Assignment(name, _) if name == target => {
                        return Ok(Value::Symbol(format!("{}::{}", first, target)));
                    }
                    _ => {}
                }
            }
        }

        Ok(Value::Symbol(parts.join("::")))
    }

    pub fn debug(&mut self, debug: bool) {
        self.debug = debug;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn run_code(code: &str) -> Result<Value, String> {
        let mut parser = Parser::new(code);
        let program = parser.parse().map_err(|e| e.message)?;
        let mut interpreter = Interpreter::new(None, Vec::new());
        interpreter.run(&program)
    }

    #[test]
    fn test_basic_arithmetic() {
        assert_eq!(run_code("5 + 3").unwrap(), Value::Integer(8));
        assert_eq!(run_code("10 - 4").unwrap(), Value::Integer(6));
        assert_eq!(run_code("6 * 7").unwrap(), Value::Integer(42));
        assert_eq!(run_code("20 / 4").unwrap(), Value::Integer(5));
        assert_eq!(run_code("17 % 5").unwrap(), Value::Integer(2));
        assert_eq!(run_code("2 ** 8").unwrap(), Value::Integer(256));
    }

    #[test]
    fn test_variables() {
        let code = r#"
            x = 10
            y = 20
            x + y
        "#;
        assert_eq!(run_code(code).unwrap(), Value::Integer(30));
    }

    #[test]
    fn test_comparisons() {
        assert_eq!(run_code("5 < 10").unwrap(), Value::Boolean(true));
        assert_eq!(run_code("10 < 5").unwrap(), Value::Boolean(false));
        assert_eq!(run_code("5 == 5").unwrap(), Value::Boolean(true));
        assert_eq!(run_code("5 != 10").unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_logical_operators() {
        assert_eq!(run_code("true && true").unwrap(), Value::Boolean(true));
        assert_eq!(run_code("true && false").unwrap(), Value::Boolean(false));
        assert_eq!(run_code("false || true").unwrap(), Value::Boolean(true));
        assert_eq!(run_code("!false").unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_if_statement() {
        let code = r#"
            x = 10
            if x > 5
                result = 100
            else
                result = 0
            end
            result
        "#;
        assert_eq!(run_code(code).unwrap(), Value::Integer(100));
    }

    #[test]
    fn test_while_loop() {
        let code = r#"
            sum = 0
            i = 1
            while i <= 5
                sum = sum + i
                i = i + 1
            end
            sum
        "#;
        assert_eq!(run_code(code).unwrap(), Value::Integer(15));
    }

    #[test]
    fn test_function_definition_and_call() {
        let code = r#"
            def add(a, b)
                return a + b
            end
            add(10, 20)
        "#;
        assert_eq!(run_code(code).unwrap(), Value::Integer(30));
    }

    #[test]
    fn test_recursive_function() {
        let code = r#"
            def factorial(n)
                if n <= 1
                    return 1
                end
                return n * factorial(n - 1)
            end
            factorial(5)
        "#;
        assert_eq!(run_code(code).unwrap(), Value::Integer(120));
    }

    #[test]
    fn test_arrays() {
        let code = r#"
            arr = [1, 2, 3, 4, 5]
            arr[2]
        "#;
        assert_eq!(run_code(code).unwrap(), Value::Integer(3));
    }

    #[test]
    fn test_hashes() {
        let code = r#"
            person = {:name => "Alice", :age => 30}
            person[:age]
        "#;
        assert_eq!(run_code(code).unwrap(), Value::Integer(30));
    }

    #[test]
    fn test_ranges() {
        let code = r#"
            range = 1..5
            range[0]
        "#;
        assert_eq!(run_code(code).unwrap(), Value::Integer(1));
    }

    #[test]
    fn test_system_instantiation() {
        let code = r#"
            system Person
                def initialize(name, age)
                    @name = name
                    @age = age
                end

                def get_age()
                    return @age
                end
            end

            person = Person.new("Alice", 25)
            person.get_age()
        "#;
        assert_eq!(run_code(code).unwrap(), Value::Integer(25));
    }

    #[test]
    fn test_system_methods() {
        let code = r#"
            system Counter
                def initialize(start)
                    @count = start
                end

                def increment()
                    @count = @count + 1
                    return @count
                end
            end

            counter = Counter.new(10)
            counter.increment()
            counter.increment()
        "#;
        assert_eq!(run_code(code).unwrap(), Value::Integer(12));
    }

    #[test]
    fn test_break_statement() {
        let code = r#"
            i = 0
            while i < 10
                if i == 5
                    break
                end
                i = i + 1
            end
            i
        "#;
        assert_eq!(run_code(code).unwrap(), Value::Integer(5));
    }

    #[test]
    fn test_bitwise_operators() {
        assert_eq!(run_code("15 & 7").unwrap(), Value::Integer(7));
        assert_eq!(run_code("8 | 4").unwrap(), Value::Integer(12));
        assert_eq!(run_code("12 ^ 5").unwrap(), Value::Integer(9));
        assert_eq!(run_code("4 << 2").unwrap(), Value::Integer(16));
        assert_eq!(run_code("16 >> 2").unwrap(), Value::Integer(4));
    }

    #[test]
    fn test_string_operations() {
        let code = r#"
            greeting = "Hello, "
            name = "World"
            greeting + name
        "#;
        match run_code(code).unwrap() {
            Value::String(s) => assert_eq!(s, "Hello, World"),
            _ => panic!("Expected string value"),
        }
    }
}
