use crate::ir::{BlockId, FuncRef, Function, InstKind, Instruction, Module, Terminator, ValueId};
use crate::syntax::ast::{BinaryOp, Literal};
use std::collections::HashMap;
use std::fmt::Write as _;

#[derive(Debug, Clone)]
pub enum Value {
    Unit,
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Record(HashMap<String, Value>),
}

impl Value {
    pub fn to_display_string(&self) -> String {
        match self {
            Value::Unit => "()".to_string(),
            Value::Int(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::String(s) => format!("\"{}\"", s),
            Value::Record(fields) => {
                let mut s = String::from("{ ");
                let mut first = true;
                for (name, value) in fields {
                    if !first {
                        s.push_str(", ");
                    }
                    first = false;
                    write!(s, "{}: {}", name, value.to_display_string()).unwrap();
                }
                s.push_str(" }");
                s
            }
        }
    }

    pub fn as_bool(&self) -> Result<bool, String> {
        match self {
            Value::Bool(b) => Ok(*b),
            _ => Err(format!("Expected bool, got {:?}", self)),
        }
    }

    pub fn as_int(&self) -> Result<i64, String> {
        match self {
            Value::Int(n) => Ok(*n),
            _ => Err(format!("Expected int, got {:?}", self)),
        }
    }

    pub fn as_float(&self) -> Result<f64, String> {
        match self {
            Value::Float(f) => Ok(*f),
            _ => Err(format!("Expected float, got {:?}", self)),
        }
    }
}

pub struct Interpreter {
    module: Module,
    output: Vec<String>,
}

impl Interpreter {
    pub fn new(module: Module) -> Self {
        Self {
            module,
            output: Vec::new(),
        }
    }

    pub fn run(&mut self) -> Result<String, String> {
        // Find main function index
        let main_idx = self
            .module
            .functions
            .iter()
            .position(|f| f.name == "main")
            .ok_or_else(|| "No main function found".to_string())?;

        // Clone the function to avoid borrow issues
        let main_func = self.module.functions[main_idx].clone();

        // Execute main
        let result = self.execute_function(&main_func, vec![])?;

        // Build output
        let mut output = String::new();
        if !self.output.is_empty() {
            writeln!(output, "=== Output ===").unwrap();
            for line in &self.output {
                writeln!(output, "{}", line).unwrap();
            }
            writeln!(output).unwrap();
        }

        writeln!(output, "=== Return Value ===").unwrap();
        writeln!(output, "{}", result.to_display_string()).unwrap();

        Ok(output)
    }

    fn execute_function(&mut self, func: &Function, args: Vec<Value>) -> Result<Value, String> {
        if args.len() != func.params.len() {
            return Err(format!(
                "Function {} expects {} arguments, got {}",
                func.name,
                func.params.len(),
                args.len()
            ));
        }

        let mut values: HashMap<ValueId, Value> = HashMap::new();

        // Bind parameters
        for (param, arg) in func.params.iter().zip(args.iter()) {
            values.insert(param.value, arg.clone());
        }

        // Execute blocks starting from block 0
        let mut current_block_id = BlockId::from_index(0);

        loop {
            let block = func
                .blocks
                .iter()
                .find(|b| b.id == current_block_id)
                .ok_or_else(|| format!("Block {:?} not found", current_block_id))?;

            // Execute instructions
            for inst in &block.instructions {
                let value = self.execute_instruction(inst, &values)?;
                values.insert(inst.id, value);
            }

            // Execute terminator
            match &block.terminator {
                Terminator::Return(val_id) => {
                    return if let Some(id) = val_id {
                        values
                            .get(id)
                            .cloned()
                            .ok_or_else(|| format!("Value {:?} not found", id))
                    } else {
                        Ok(Value::Unit)
                    };
                }
                Terminator::Jump(next_block) => {
                    current_block_id = *next_block;
                }
                Terminator::Branch {
                    condition,
                    then_block,
                    else_block,
                } => {
                    let cond_val = values
                        .get(condition)
                        .ok_or_else(|| format!("Condition value {:?} not found", condition))?;
                    let cond = cond_val.as_bool()?;
                    current_block_id = if cond { *then_block } else { *else_block };
                }
            }
        }
    }

    fn execute_instruction(
        &mut self,
        inst: &Instruction,
        values: &HashMap<ValueId, Value>,
    ) -> Result<Value, String> {
        match &inst.kind {
            InstKind::Literal(lit) => self.execute_literal(lit),
            InstKind::Binary { op, lhs, rhs } => {
                let lhs_val = values
                    .get(lhs)
                    .ok_or_else(|| format!("LHS value {:?} not found", lhs))?;
                let rhs_val = values
                    .get(rhs)
                    .ok_or_else(|| format!("RHS value {:?} not found", rhs))?;
                self.execute_binary_op(*op, lhs_val, rhs_val)
            }
            InstKind::Call { func, args } => {
                let arg_values: Result<Vec<_>, _> = args
                    .iter()
                    .map(|id| {
                        values
                            .get(id)
                            .cloned()
                            .ok_or_else(|| format!("Argument value {:?} not found", id))
                    })
                    .collect();
                self.execute_call(func, arg_values?)
            }
            InstKind::Record {
                type_path: _,
                fields,
            } => {
                let mut record = HashMap::new();
                for (name, val_id) in fields {
                    let value = values
                        .get(val_id)
                        .cloned()
                        .ok_or_else(|| format!("Field value {:?} not found", val_id))?;
                    record.insert(name.clone(), value);
                }
                Ok(Value::Record(record))
            }
            InstKind::Path(_path) => {
                // For now, return a placeholder
                Ok(Value::Unit)
            }
            InstKind::Phi { incomings } => {
                // For simple cases, just take the first value
                if let Some((_, val_id)) = incomings.first() {
                    values
                        .get(val_id)
                        .cloned()
                        .ok_or_else(|| format!("Phi value {:?} not found", val_id))
                } else {
                    Ok(Value::Unit)
                }
            }
        }
    }

    fn execute_literal(&self, lit: &Literal) -> Result<Value, String> {
        match lit {
            Literal::Unit => Ok(Value::Unit),
            Literal::Int(n) => Ok(Value::Int(*n)),
            Literal::Float(f) => Ok(Value::Float(*f)),
            Literal::Bool(b) => Ok(Value::Bool(*b)),
            Literal::String(s) => Ok(Value::String(s.clone())),
        }
    }

    fn execute_binary_op(&self, op: BinaryOp, lhs: &Value, rhs: &Value) -> Result<Value, String> {
        match (op, lhs, rhs) {
            // Integer operations
            (BinaryOp::Add, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (BinaryOp::Sub, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (BinaryOp::Mul, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (BinaryOp::Div, Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::Int(a / b))
                }
            }
            (BinaryOp::Mod, Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err("Modulo by zero".to_string())
                } else {
                    Ok(Value::Int(a % b))
                }
            }

            // Float operations
            (BinaryOp::Add, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (BinaryOp::Sub, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (BinaryOp::Mul, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (BinaryOp::Div, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),

            // Comparison operations
            (BinaryOp::Eq, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a == b)),
            (BinaryOp::Ne, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a != b)),
            (BinaryOp::Lt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
            (BinaryOp::Le, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
            (BinaryOp::Gt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
            (BinaryOp::Ge, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),

            (BinaryOp::Eq, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a == b)),
            (BinaryOp::Ne, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a != b)),

            // Logical operations
            (BinaryOp::And, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a && *b)),
            (BinaryOp::Or, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a || *b)),

            _ => Err(format!(
                "Unsupported binary operation: {:?} {:?} {:?}",
                lhs, op, rhs
            )),
        }
    }

    fn execute_call(&mut self, func: &FuncRef, args: Vec<Value>) -> Result<Value, String> {
        match func {
            FuncRef::Function(path) => {
                let func_name = path.segments.last().map(|s| s.as_str()).unwrap_or("");

                // Handle built-in functions
                match func_name {
                    "println" => {
                        let output = if args.is_empty() {
                            String::new()
                        } else {
                            args[0].to_display_string()
                        };
                        self.output.push(output);
                        Ok(Value::Unit)
                    }
                    "print" => {
                        let output = if args.is_empty() {
                            String::new()
                        } else {
                            args[0].to_display_string()
                        };
                        self.output.push(output);
                        Ok(Value::Unit)
                    }
                    _ => {
                        // Try to find the function in the module
                        let target_func = self
                            .module
                            .functions
                            .iter()
                            .find(|f| f.name == func_name)
                            .ok_or_else(|| format!("Function {} not found", func_name))?
                            .clone();

                        self.execute_function(&target_func, args)
                    }
                }
            }
            FuncRef::Method(method_name) => {
                Err(format!("Method calls not yet supported: {}", method_name))
            }
        }
    }
}

impl BlockId {
    pub fn from_index(index: u32) -> Self {
        BlockId(index)
    }
}
