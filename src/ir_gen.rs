use std::collections::HashMap;
use crate::llvm::{IRBuilder, FnValue, FunctionPassManager, Module, Value}
use crate::parser::{ExprAST, FunctionAST, PrototypeAST}
use crate::Either;

type ParseResult<T> = Result<T, String>;

pub struct IRGen<'llvm, 'a'> {
    builder: &'a IRBuilder<'llvm>,
    module: &'llvm Module,
    fn_proto_map: &'a mut HashMap<String, PrototypeAST>,
    fpm: &'a FunctionPassManager<'llvm>,
}

impl<'llvm, 'a> IRGen<'llvm, 'a> {

    pub fn compile(
        module: &'llvm Module,
        fn_proto_map: &mut HashMap<String, PrototypeAST>,
        compilee: Either<&PrototypeAST, &FunctionAST>,
    ) -> IRGenResult<FnValue<'llvm>> {
        let builder = IRBuilder::new(module);
        let fpm = FunctionPassManager::new(module);
        let mut ir_gen = IRGen {
            builder: &builder,
            module: module,
            fn_proto_map: fn_proto_map,
            fpm: &fpm,
        };

        match compilee {
            Either::Left(proto) => ir_gen.compile_proto(proto),
            Either::Right(func) => ir_gen.compile_func(func),
        }
    }

    fn irgen_expr(
        &self,
        expr: &ExprAST,
        named_values: &mut HashMap<String, Value<'llvm>>,
    ) -> IRGenResult<Value<'llvm>> {
        match expr {
            ExprAST::Number(value) => Ok(self.builder.build_const_f64(*value)),
            ExprAST::Variable(name) => {
                if let Some(value) = named_values.get(name) {
                    Ok(*value)
                } else {
                    Err(format!("Unknown variable name: {}", name))
                }
            },
            ExprAST::BinaryOp(op, lhs, rhs) => {
                let lhs = self.irgen_expr(lhs, named_values)?;
                let rhs = self.irgen_expr(rhs, named_values)?;
                match op.as_str() {
                    "+" => Ok(self.builder.build_add(lhs, rhs)),
                    "-" => Ok(self.builder.build_sub(lhs, rhs)),
                    "*" => Ok(self.builder.build_mul(lhs, rhs)),
                    "/" => Ok(self.builder.build_div(lhs, rhs)),
                    "<" => Ok(self.builder.build_lt(lhs, rhs)),
                    ">" => Ok(self.builder.build_gt(lhs, rhs)),
                    "<=" => Ok(self.builder.build_le(lhs, rhs)),
                    ">=" => Ok(self.builder.build_ge(lhs, rhs)),
                    "==" => Ok(self.builder.build_eq(lhs, rhs)),
                    "!=" => Ok(self.builder.build_ne(lhs, rhs)),
                    _ => Err(format!("Unknown binary operator: {}", op)),
                }
            },
            ExprAST::Call(callee, args) => {
                let callee = match self.fn_proto_map.get(callee) {
                    Some(proto) => proto,
                    None => return Err(format!("Unknown function referenced: {}", callee)),
                };

                if callee.args.len() != args.len() {
                    return Err(format!("Incorrect # of arguments passed to {}: expected {}, found {}", callee.name, callee.args.len(), args.len()));
                }

                let mut args_values = Vec::new();
                for arg in args {
                    args_values.push(self.irgen_expr(arg, named_values)?);
                }

                Ok(self.builder.build_call(callee.name, args_values))
            },
            ExprAST::If { condition, then, else_ } => {
                let condition = self.irgen_expr(condition, named_values)?;
                let zero = self.builder.build_const_f64(0.0);
                let condition = self.builder.build_fcmp_gt(condition, zero);
                let function = self.builder.get_insert_block().get_parent();
                let then_block = self.builder.build_append_block("then", function);
                let else_block = self.builder.build_append_block("else", function);
                let merge_block = self.builder.build_append_block("ifcont", function);
                self.builder.build_cond_br(condition, then_block, else_block);

                self.builder.set_insert_point(then_block);
                let then_value = self.irgen_expr(then, named_values)?;
                self.builder.build_br(merge_block);
                let then_block = self.builder.get_insert_block();

                self.builder.set_insert_point(else_block);
                let else_value = self.irgen_expr(else_, named_values)?;
                self.builder.build_br(merge_block);
                let else_block = self.builder.get_insert_block();

                self.builder.set_insert_point(merge_block);
                let phi = self.builder.build_phi(then_value.get_type());
                phi.add_incoming(then_value, then_block);
                phi.add_incoming(else_value, else_block);
                Ok(phi)
            },
            ExprAST::For {
                var,
                start,
                end,
                step,
                body,
            } => {

                let start = self.irgen_expr(start, named_values)?;
                let function = self.builder.get_insert_block().get_parent();
                let entry_block = self.builder.get_insert_block();
                let loop_block = self.module.append_basic_block(function);

                self.builder.build_br(loop_block);
                self.builder.position_at_end(loop_block);

                let variable = self.builder.phi(self.module.type_f64(), &[(start, entry_block)]);
                let old_value = named_values.insert(var.clone(), variable);

                self.irgen_expr(body, named_values)?;

                let step = self.irgen_expr(step, named_values)?;
                let next_var = self.builder.build_add(variable, step);

                let end = self.irgen_expr(end, named_values)?;
                let end_cond = self.builder.build_fcmp_gt(end, next_var);
                let loop_end_block = self.builder.get_insert_block();
                let after_block = self.module.append_basic_block(function);

                self.builder.build_cond_br(end_cond, loop_block, after_block);
                self.builder.position_at_end(after_block);

                if let Some(old_value) = old_value {
                    named_values.insert(var.clone(), old_value);
                } else {
                    named_values.remove(&var);
                }

                Ok(self.builder.build_const_f64(0.0))
            }
        }
    }

    fn irgen_proto(&self, PrototypeAST(name, args): &PrototypeAST) -> FnValue<'llvm> {
        let type_f64 = self.module.type_f64();
        let mut doubles = Vec::new();

        let function_type = self.module.type_function(type_f64, &doubles);
        let function = self.module.add_function(name, function_type);

        for i in 0..function.args() {
            function.arg(i).set_name(&args[i]);
        }
        function
    }

    fn irgen_function(
        &mut self,
        FunctionAST(proto, body): &FunctionAST,
        named_values: &mut HashMap<String, Value<'llvm>>,
    ) -> IRGenResult<FnValue<'llvm>> {

        self.fn_proto_map.insert(proto.name.clone(), proto.clone());
        let function = self.get_function(proto.name.as_str());

        if function.count_basic_blocks() != 0 {
            return Err(format!("Redefinition of function {}", proto.name));
        }

        let basic_block = self.module.append_basic_block(function);
        self.builder.build_br(basic_block);

        named_values.clear();

        for i in 0..function.args() {
            let arg = function.arg(i);
            named_values.insert(arg.get_name().into(), arg);
        }

        if let Ok(ret) = self.irgen_expr(body, named_values) {
            self.builder.build_ret(ret);
            self.fpm.run_on(function);
            Ok(function)
        } else {
            function.delete();
            Err(format!("Error generating code for function {}", proto.name))
        }
    }

    fn get_function(&self, name: &str) -> Option<FnValue<'llvm>> {
        let callee = self.module.get_function(name);
        if callee.count_basic_blocks() != 0 {
            Some(callee)
        } else {
            None
        }
    }
}