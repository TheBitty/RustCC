                        if let Statement::Block(block_statements) = init.as_mut() {
                            self.inline_function_calls(block_statements, inline_candidates);
                        } 