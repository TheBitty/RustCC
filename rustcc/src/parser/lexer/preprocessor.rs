// preprocessor.rs
// Handling of preprocessor directives

use crate::parser::lexer::utils::{is_alpha, is_alphanumeric, is_digit, is_whitespace};
use crate::parser::lexer::Lexer;
use crate::parser::token::TokenType;

impl Lexer {
    /// Handles a preprocessor directive
    pub(crate) fn preprocessor_directive(&mut self) {
        // Add the # token
        self.add_token(TokenType::Hash);

        // Skip whitespace
        while self.peek() == ' ' || self.peek() == '\t' {
            self.advance();
        }

        // Mark the start of the directive name
        self.start = self.current;

        // Read the directive name
        while is_alpha(self.peek()) {
            self.advance();
        }

        // Get the directive name
        let directive = self.source[self.start..self.current].to_string();

        // Check if it's a known preprocessor directive
        match directive.as_str() {
            "include" => {
                self.add_token(TokenType::PPInclude);
                self.handle_include_directive();
            }
            "define" => {
                self.add_token(TokenType::PPDefine);
                self.handle_define_directive();
            }
            "undef" => {
                self.add_token(TokenType::PPUndef);
                self.handle_undef_directive();
            }
            "ifdef" => {
                self.add_token(TokenType::PPIfDef);
                self.handle_ifdef_directive();
            }
            "ifndef" => {
                self.add_token(TokenType::PPIfNDef);
                self.handle_ifndef_directive();
            }
            "if" => {
                self.add_token(TokenType::PPIf);
                self.handle_if_directive();
            }
            "else" => self.add_token(TokenType::PPElse),
            "elif" => {
                self.add_token(TokenType::PPElif);
                self.handle_elif_directive();
            }
            "endif" => self.add_token(TokenType::PPEndif),
            "pragma" => {
                self.add_token(TokenType::PPPragma);
                self.handle_pragma_directive();
            }
            "error" => {
                self.add_token(TokenType::PPErrorDir);
                self.handle_error_directive();
            }
            "warning" => {
                self.add_token(TokenType::PPWarning);
                self.handle_warning_directive();
            }
            "line" => {
                self.add_token(TokenType::PPLine);
                self.handle_line_directive();
            }
            // C11 _Pragma operator is handled differently (as a token, not a directive)
            // C23 embed and elifdef/elifndef directives
            "embed" => {
                // C23 feature for embedding binary files
                self.add_token(TokenType::PPEmbed);
                self.handle_embed_directive();
            }
            "elifdef" => {
                // C23 feature combining elif and ifdef
                self.add_token(TokenType::PPElifDef);
                self.handle_elifdef_directive();
            }
            "elifndef" => {
                // C23 feature combining elif and ifndef
                self.add_token(TokenType::PPElifNDef);
                self.handle_elifndef_directive();
            }
            _ => {
                // Unknown preprocessor directive
                // C standard says to ignore unknown directives
                // We'll just skip to the end of the line
                while !self.is_at_end() && self.peek() != '\n' {
                    self.advance();
                }
            }
        }
    }

    /// Handles an #include directive
    pub(crate) fn handle_include_directive(&mut self) {
        // Skip whitespace
        while is_whitespace(self.peek()) {
            self.advance();
        }

        // Check for include syntax: #include <file> or #include "file"
        if self.peek() == '<' {
            // System include: <file>
            self.advance(); // Consume '<'
            self.start = self.current;

            // Read until '>' or end of line
            while !self.is_at_end() && self.peek() != '>' && self.peek() != '\n' {
                self.advance();
            }

            // Get the include path
            let include_path = self.source[self.start..self.current].to_string();
            
            // Store the include path for later use
            self.includes.push(include_path);

            // Consume the closing '>'
            if !self.is_at_end() && self.peek() == '>' {
                self.advance();
            }
        } else if self.peek() == '"' {
            // Local include: "file"
            self.advance(); // Consume '"'
            self.start = self.current;

            // Read until '"' or end of line
            while !self.is_at_end() && self.peek() != '"' && self.peek() != '\n' {
                self.advance();
            }

            // Get the include path
            let include_path = self.source[self.start..self.current].to_string();
            
            // Store the include path for later use
            self.includes.push(include_path);

            // Consume the closing '"'
            if !self.is_at_end() && self.peek() == '"' {
                self.advance();
            }
        } else if is_alpha(self.peek()) {
            // Macro-based include: #include MACRO
            // or computed include: #include MACRO(args)
            self.start = self.current;
            
            // Read the macro name
            while is_alphanumeric(self.peek()) || self.peek() == '_' {
                self.advance();
            }
            
            // Handle macro with arguments
            if self.peek() == '(' {
                // Process macro invocation with arguments
                self.process_macro_invocation();
            } else {
                // Simple macro name
                let macro_name = self.source[self.start..self.current].to_string();
                // In a real implementation, we would expand the macro here
            }
        }

        // Skip to the end of the line
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
    }

    /// Handles a #define directive
    pub(crate) fn handle_define_directive(&mut self) {
        // Skip whitespace
        while is_whitespace(self.peek()) {
            self.advance();
        }

        // Read the macro name
        self.start = self.current;
        if is_alpha(self.peek()) || self.peek() == '_' {
            while is_alphanumeric(self.peek()) || self.peek() == '_' {
                self.advance();
            }
        } else {
            // Invalid macro name
            // Skip to the end of the line
            while !self.is_at_end() && self.peek() != '\n' {
                self.advance();
            }
            return;
        }

        // Get the macro name
        let macro_name = self.source[self.start..self.current].to_string();

        // Check if it's a function-like macro
        let is_function_like = self.peek() == '(';
        let mut parameters = Vec::new();

        if is_function_like {
            self.advance(); // Consume '('

            // Parse parameters
            if self.peek() != ')' {
                loop {
                    // Skip whitespace
                    while is_whitespace(self.peek()) {
                        self.advance();
                    }

                    // Read parameter name
                    self.start = self.current;
                    while is_alphanumeric(self.peek()) || self.peek() == '_' {
                        self.advance();
                    }

                    // Get the parameter name
                    let param = self.source[self.start..self.current].to_string();
                    if !param.is_empty() {
                        parameters.push(param);
                    }

                    // Skip whitespace
                    while is_whitespace(self.peek()) {
                        self.advance();
                    }

                    // Check for variadic macro
                    if self.peek() == '.' && self.peek_next() == '.' {
                        // Consume "..."
                        self.advance();
                        self.advance();
                        self.advance();
                        parameters.push("...".to_string());
                        break;
                    }

                    // Check for end of parameters
                    if self.peek() == ')' {
                        break;
                    }

                    // Expect a comma
                    if self.peek() == ',' {
                        self.advance();
                    } else {
                        // Invalid parameter list
                        // Skip to the end of the line
                        while !self.is_at_end() && self.peek() != '\n' {
                            self.advance();
                        }
                        return;
                    }
                }
            }

            // Consume the closing ')'
            if self.peek() == ')' {
                self.advance();
            }
        }

        // Skip whitespace
        while is_whitespace(self.peek()) {
            self.advance();
        }

        // Read the replacement text
        self.start = self.current;
        let mut replacement = String::new();
        let mut in_string = false;
        let mut in_char = false;
        let mut escaped = false;

        while !self.is_at_end() && self.peek() != '\n' {
            let c = self.peek();
            
            // Handle string literals
            if c == '"' && !escaped {
                in_string = !in_string;
            }
            
            // Handle character literals
            if c == '\'' && !escaped {
                in_char = !in_char;
            }
            
            // Handle escape sequences
            if c == '\\' && !escaped {
                escaped = true;
            } else {
                escaped = false;
            }
            
            // Handle line continuation
            if c == '\\' && self.peek_next() == '\n' {
                self.advance(); // Consume '\'
                self.advance(); // Consume '\n'
                self.line += 1;
                self.column = 1;
                self.at_line_start = true;
                continue;
            }
            
            // Add character to replacement text
            replacement.push(c);
            self.advance();
        }

        // Store the macro definition
        // In a real implementation, we would store this in a macro table
        // For now, we'll just acknowledge it
        if is_function_like {
            // Function-like macro
            // self._defines.insert(macro_name, (parameters, replacement));
        } else {
            // Object-like macro
            // self._defines.insert(macro_name, replacement);
        }
    }

    /// Process a macro invocation with arguments
    fn process_macro_invocation(&mut self) {
        // Consume the opening '('
        self.advance();
        
        // Parse arguments
        let mut args = Vec::new();
        let mut current_arg = String::new();
        let mut paren_depth = 1;
        let mut in_string = false;
        let mut in_char = false;
        let mut escaped = false;
        
        while !self.is_at_end() && paren_depth > 0 {
            let c = self.peek();
            
            // Handle nested parentheses
            if !in_string && !in_char {
                if c == '(' {
                    paren_depth += 1;
                } else if c == ')' {
                    paren_depth -= 1;
                    if paren_depth == 0 {
                        // End of arguments
                        if !current_arg.is_empty() {
                            args.push(current_arg.trim().to_string());
                        }
                        break;
                    }
                } else if c == ',' && paren_depth == 1 {
                    // Argument separator
                    args.push(current_arg.trim().to_string());
                    current_arg = String::new();
                    self.advance();
                    continue;
                }
            }
            
            // Handle string literals
            if c == '"' && !escaped {
                in_string = !in_string;
            }
            
            // Handle character literals
            if c == '\'' && !escaped {
                in_char = !in_char;
            }
            
            // Handle escape sequences
            if c == '\\' && !escaped {
                escaped = true;
            } else {
                escaped = false;
            }
            
            // Add character to current argument
            if paren_depth > 0 {
                current_arg.push(c);
            }
            
            self.advance();
        }
        
        // Consume the closing ')'
        if !self.is_at_end() && self.peek() == ')' {
            self.advance();
        }
        
        // In a real implementation, we would expand the macro with these arguments
    }

    /// Process a macro replacement
    pub(crate) fn process_macro_replacement(&mut self) {
        // This is a simplified implementation
        // In a real compiler, we would:
        // 1. Look up the macro in the macro table
        // 2. If it's a function-like macro, parse the arguments
        // 3. Perform token pasting (##) and stringification (#)
        // 4. Replace parameters with arguments
        // 5. Recursively expand macros in the replacement text
        
        // For now, we'll just skip the macro name and any arguments
        self.start = self.current;
        
        // Read the macro name
        while is_alphanumeric(self.peek()) || self.peek() == '_' {
            self.advance();
        }
        
        // Check for function-like macro invocation
        if self.peek() == '(' {
            self.process_macro_invocation();
        }
    }

    /// Handles a #undef directive
    pub(crate) fn handle_undef_directive(&mut self) {
        // Skip whitespace
        while is_whitespace(self.peek()) {
            self.advance();
        }

        // Read the macro name
        self.start = self.current;
        if is_alpha(self.peek()) || self.peek() == '_' {
            while is_alphanumeric(self.peek()) || self.peek() == '_' {
                self.advance();
            }
        } else {
            // Invalid macro name
            // Skip to the end of the line
            while !self.is_at_end() && self.peek() != '\n' {
                self.advance();
            }
            return;
        }

        // Get the macro name
        let macro_name = self.source[self.start..self.current].to_string();

        // Remove the macro from the defines table
        // In a real implementation, we would do this
        // self._defines.remove(&macro_name);

        // Skip to the end of the line
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
    }

    /// Handles an #ifdef directive
    pub(crate) fn handle_ifdef_directive(&mut self) {
        // Skip whitespace
        while is_whitespace(self.peek()) {
            self.advance();
        }

        // Read the macro name
        self.start = self.current;
        if is_alpha(self.peek()) || self.peek() == '_' {
            while is_alphanumeric(self.peek()) || self.peek() == '_' {
                self.advance();
            }
        } else {
            // Invalid macro name
            // Skip to the end of the line
            while !self.is_at_end() && self.peek() != '\n' {
                self.advance();
            }
            return;
        }

        // Get the macro name
        let macro_name = self.source[self.start..self.current].to_string();

        // In a real implementation, we would check if the macro is defined
        // and conditionally include/exclude code based on that

        // Skip to the end of the line
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
    }

    /// Handles an #ifndef directive
    pub(crate) fn handle_ifndef_directive(&mut self) {
        // Skip whitespace
        while is_whitespace(self.peek()) {
            self.advance();
        }

        // Read the macro name
        self.start = self.current;
        if is_alpha(self.peek()) || self.peek() == '_' {
            while is_alphanumeric(self.peek()) || self.peek() == '_' {
                self.advance();
            }
        } else {
            // Invalid macro name
            // Skip to the end of the line
            while !self.is_at_end() && self.peek() != '\n' {
                self.advance();
            }
            return;
        }

        // Get the macro name
        let macro_name = self.source[self.start..self.current].to_string();

        // In a real implementation, we would check if the macro is not defined
        // and conditionally include/exclude code based on that

        // Skip to the end of the line
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
    }

    /// Handles an #if directive
    pub(crate) fn handle_if_directive(&mut self) {
        // Skip whitespace
        while is_whitespace(self.peek()) {
            self.advance();
        }

        // Read the condition expression
        self.start = self.current;
        
        // In a real implementation, we would parse and evaluate the condition
        // For now, we'll just skip to the end of the line
        
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
    }

    /// Handles an #elif directive
    pub(crate) fn handle_elif_directive(&mut self) {
        // Skip whitespace
        while is_whitespace(self.peek()) {
            self.advance();
        }

        // Read the condition expression
        self.start = self.current;
        
        // In a real implementation, we would parse and evaluate the condition
        // For now, we'll just skip to the end of the line
        
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
    }

    /// Handles a #pragma directive
    pub(crate) fn handle_pragma_directive(&mut self) {
        // Skip whitespace
        while is_whitespace(self.peek()) {
            self.advance();
        }

        // Read the pragma name
        self.start = self.current;
        while is_alpha(self.peek()) {
            self.advance();
        }

        // Get the pragma name
        let pragma_name = self.source[self.start..self.current].to_string();

        // In a real implementation, we would handle specific pragmas
        // For now, we'll just skip to the end of the line

        // Skip to the end of the line
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
    }

    /// Handles an #error directive
    pub(crate) fn handle_error_directive(&mut self) {
        // Skip whitespace
        while is_whitespace(self.peek()) {
            self.advance();
        }

        // Read the error message
        self.start = self.current;
        
        // In a real implementation, we would report this as a compilation error
        // For now, we'll just skip to the end of the line
        
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
    }

    /// Handles a #warning directive
    pub(crate) fn handle_warning_directive(&mut self) {
        // Skip whitespace
        while is_whitespace(self.peek()) {
            self.advance();
        }

        // Read the warning message
        self.start = self.current;
        
        // In a real implementation, we would report this as a compilation warning
        // For now, we'll just skip to the end of the line
        
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
    }

    /// Handles a #line directive
    pub(crate) fn handle_line_directive(&mut self) {
        // Skip whitespace
        while is_whitespace(self.peek()) {
            self.advance();
        }

        // Read the line number
        self.start = self.current;
        while is_digit(self.peek()) {
            self.advance();
        }

        // Get the line number
        if self.start < self.current {
            let line_str = self.source[self.start..self.current].to_string();
            if let Ok(line_num) = line_str.parse::<usize>() {
                // In a real implementation, we would update the line number
                // self.line = line_num;
            }
        }

        // Skip whitespace
        while is_whitespace(self.peek()) {
            self.advance();
        }

        // Check for optional filename
        if self.peek() == '"' {
            self.advance(); // Consume '"'
            self.start = self.current;

            // Read until '"' or end of line
            while !self.is_at_end() && self.peek() != '"' && self.peek() != '\n' {
                self.advance();
            }

            // Get the filename
            let filename = self.source[self.start..self.current].to_string();
            
            // In a real implementation, we would update the current filename
            
            // Consume the closing '"'
            if !self.is_at_end() && self.peek() == '"' {
                self.advance();
            }
        }

        // Skip to the end of the line
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
    }

    /// Handles a #embed directive (C23)
    pub(crate) fn handle_embed_directive(&mut self) {
        // Skip whitespace
        while is_whitespace(self.peek()) {
            self.advance();
        }

        // C23 #embed directive allows embedding binary files
        // Format: #embed [prefix-list] resource-path [suffix-list]
        
        // For now, we'll just skip to the end of the line
        // In a real implementation, we would parse the resource path and embed the file
        
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
    }

    /// Handles an #elifdef directive (C23)
    pub(crate) fn handle_elifdef_directive(&mut self) {
        // Skip whitespace
        while is_whitespace(self.peek()) {
            self.advance();
        }

        // Read the macro name
        self.start = self.current;
        if is_alpha(self.peek()) || self.peek() == '_' {
            while is_alphanumeric(self.peek()) || self.peek() == '_' {
                self.advance();
            }
        } else {
            // Invalid macro name
            // Skip to the end of the line
            while !self.is_at_end() && self.peek() != '\n' {
                self.advance();
            }
            return;
        }

        // Get the macro name
        let macro_name = self.source[self.start..self.current].to_string();

        // In a real implementation, we would check if the macro is defined
        // and conditionally include/exclude code based on that

        // Skip to the end of the line
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
    }

    /// Handles an #elifndef directive (C23)
    pub(crate) fn handle_elifndef_directive(&mut self) {
        // Skip whitespace
        while is_whitespace(self.peek()) {
            self.advance();
        }

        // Read the macro name
        self.start = self.current;
        if is_alpha(self.peek()) || self.peek() == '_' {
            while is_alphanumeric(self.peek()) || self.peek() == '_' {
                self.advance();
            }
        } else {
            // Invalid macro name
            // Skip to the end of the line
            while !self.is_at_end() && self.peek() != '\n' {
                self.advance();
            }
            return;
        }

        // Get the macro name
        let macro_name = self.source[self.start..self.current].to_string();

        // In a real implementation, we would check if the macro is not defined
        // and conditionally include/exclude code based on that

        // Skip to the end of the line
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
    }
}
