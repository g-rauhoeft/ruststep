use super::{expression::*, identifier::*, types::*, util::*};

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Alias {
        name: String,
        dest: String,
        qualifiers: Vec<Qualifier>,
        statements: Vec<Statement>,
    },

    Assignment {
        name: String,
        qualifiers: Vec<Qualifier>,
        expr: Expression,
    },

    Compound {
        statements: Vec<Statement>,
    },

    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },

    Case {
        selector: Expression,
        actions: Vec<(Vec<Expression>, Statement)>,
        otherwise: Option<Box<Statement>>,
    },

    Repeat {
        control: RepeatControl,
        statements: Vec<Statement>,
    },

    Return {
        value: Option<Expression>,
    },

    ProcedureCall {
        procedure: Procedure,
        parameters: Option<Vec<Expression>>,
    },

    Skip,
    Escape,
    Null,
}

/// 309 stmt = [alias_stmt] | [assignment_stmt] | [case_stmt] | [compound_stmt] | [escape_stmt] | [if_stmt] | [null_stmt] | [procedure_call_stmt] | [repeat_stmt] | [return_stmt] | [skip_stmt] .
pub fn stmt(input: &str) -> ParseResult<Statement> {
    alt((
        alias_stmt,
        assignment_stmt,
        case_stmt,
        compound_stmt,
        escape_stmt,
        if_stmt,
        null_stmt,
        procedure_call_stmt,
        repeat_stmt,
        return_stmt,
        skip_stmt,
    ))
    .parse(input)
}

/// 174 alias_stmt = ALIAS [variable_id] FOR [general_ref] { [qualifier] } `;` [stmt] { [stmt] } END_ALIAS `;` .
pub fn alias_stmt(input: &str) -> ParseResult<Statement> {
    tuple((
        tag("ALIAS"),
        variable_id,
        tag("FOR"),
        general_ref,
        spaced_many0(qualifier),
        char(';'),
        space_separated(stmt),
        tag("END_ASLIAS"),
        char(';'),
    ))
    .map(
        |(_alias, name, _for, dest, qualifiers, _semicolon1, statements, _end, _semicolon2)| {
            Statement::Alias {
                name,
                dest,
                qualifiers,
                statements,
            }
        },
    )
    .parse(input)
}

/// 176 assignment_stmt = [general_ref] { [qualifier] } `:=` [expression] `;` .
pub fn assignment_stmt(input: &str) -> ParseResult<Statement> {
    tuple((
        general_ref,
        spaced_many0(qualifier),
        tag(":="),
        expression,
        char(';'),
    ))
    .map(
        |(name, qualifiers, _def, expr, _semicolon)| Statement::Assignment {
            name,
            qualifiers,
            expr,
        },
    )
    .parse(input)
}

/// 191 case_stmt = CASE [selector] OF { [case_action] } \[ OTHERWISE `:` [stmt] \] END_CASE `;` .
pub fn case_stmt(input: &str) -> ParseResult<Statement> {
    tuple((
        tag("CASE"),
        selector,
        tag("OF"),
        spaced_many0(case_action),
        opt(tuple((tag("OTHERWISE"), char(':'), stmt))
            .map(|(_otherwise, _colon, stmt)| Box::new(stmt))),
        tag("END_CASE"),
        char(';'),
    ))
    .map(
        |(_case, selector, _of, actions, otherwise, _end, _semicolon)| Statement::Case {
            selector,
            actions,
            otherwise,
        },
    )
    .parse(input)
}

/// 299 selector = [expression] .
pub fn selector(input: &str) -> ParseResult<Expression> {
    expression(input)
}

/// 189 case_action = [case_label] { `,` [case_label] } `:` [stmt] .
pub fn case_action(input: &str) -> ParseResult<(Vec<Expression>, Statement)> {
    tuple((comma_separated(case_label), char(':'), stmt))
        .map(|(labels, _colon, statement)| (labels, statement))
        .parse(input)
}

/// 190 case_label = [expression] .
pub fn case_label(input: &str) -> ParseResult<Expression> {
    expression(input)
}

/// 192 compound_stmt = BEGIN [stmt] { [stmt] } END `;` .
pub fn compound_stmt(input: &str) -> ParseResult<Statement> {
    tuple((tag("BEGIN"), space_separated(stmt), tag("END"), char(';')))
        .map(|(_begin, statements, _end, _semicolon)| Statement::Compound { statements })
        .parse(input)
}

/// 214 escape_stmt = ESCAPE `;` .
pub fn escape_stmt(input: &str) -> ParseResult<Statement> {
    tuple((tag("ESCAPE"), char(';')))
        .map(|(_skip, _semicolon)| Statement::Escape)
        .parse(input)
}

/// 233 if_stmt = IF [logical_expression] THEN [stmt] { [stmt] } \[ ELSE [stmt] { [stmt] } \] END_IF `;` .
pub fn if_stmt(input: &str) -> ParseResult<Statement> {
    tuple((
        tag("IF"),
        logical_expression,
        tag("THEN"),
        space_separated(stmt),
        opt(tuple((tag("ELSE"), space_separated(stmt))).map(|(_else, stmts)| stmts)),
        tag("END_IF"),
        char(';'),
    ))
    .map(
        |(_if, condition, _then, then_branch, else_branch, _endif, _semicolon)| Statement::If {
            condition,
            then_branch,
            else_branch,
        },
    )
    .parse(input)
}

/// 260 null_stmt = `;` .
pub fn null_stmt(input: &str) -> ParseResult<Statement> {
    char(';').map(|_c| Statement::Null).parse(input)
}

/// 270 procedure_call_stmt = ( [built_in_procedure] | [procedure_ref] ) \[ [actual_parameter_list] \] `;` .
pub fn procedure_call_stmt(input: &str) -> ParseResult<Statement> {
    tuple((
        alt((
            built_in_procedure,
            procedure_ref.map(|name| Procedure::Reference(name)),
        )),
        opt(actual_parameter_list),
        char(';'),
    ))
    .map(
        |(procedure, parameters, _semicolon)| Statement::ProcedureCall {
            procedure,
            parameters,
        },
    )
    .parse(input)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Procedure {
    Reference(String),
    /// Built-in procedure `INSERT`
    Insert,
    /// Built-in procedure `REMOVE`
    Remove,
}

/// 188 built_in_procedure = INSERT | REMOVE .
pub fn built_in_procedure(input: &str) -> ParseResult<Procedure> {
    alt((
        tag("INSERT").map(|_| Procedure::Insert),
        tag("REMOVE").map(|_| Procedure::Remove),
    ))
    .parse(input)
}

/// 286 repeat_stmt = REPEAT [repeat_control] `;` [stmt] { [stmt] } END_REPEAT `;` .
pub fn repeat_stmt(input: &str) -> ParseResult<Statement> {
    tuple((
        tag("REPEAT"),
        repeat_control,
        char(';'),
        space_separated(stmt),
        tag("END_REPEAT"),
        char(';'),
    ))
    .map(
        |(_repeat, control, _semicolon1, statements, _end, _semicolon2)| Statement::Repeat {
            control,
            statements,
        },
    )
    .parse(input)
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepeatControl {
    pub increment: Option<RepeatIncrement>,
    pub while_: Option<Expression>,
    pub until: Option<Expression>,
}

/// 285 repeat_control = \[ [increment_control] \] \[ [while_control] \] \[ [until_control] \] .
pub fn repeat_control(input: &str) -> ParseResult<RepeatControl> {
    tuple((
        opt(increment_control),
        opt(while_control),
        opt(until_control),
    ))
    .map(|(increment, while_, until)| RepeatControl {
        increment,
        while_,
        until,
    })
    .parse(input)
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepeatIncrement {
    pub variable: String,
    pub begin: Expression,
    pub end: Expression,
    pub increment: Option<Expression>,
}

/// 235 increment_control = [variable_id] `:=` [bound_1] TO [bound_2] \[ BY [increment] \] .
pub fn increment_control(input: &str) -> ParseResult<RepeatIncrement> {
    tuple((
        variable_id,
        tag(":="),
        bound_1,
        tag("TO"),
        bound_2,
        opt(tuple((tag("BY"), increment)).map(|(_by, inc)| inc)),
    ))
    .map(
        |(variable, _def, begin, _to, end, increment)| RepeatIncrement {
            variable,
            begin,
            end,
            increment,
        },
    )
    .parse(input)
}

/// 234 increment = [numeric_expression] .
pub fn increment(input: &str) -> ParseResult<Expression> {
    numeric_expression(input)
}

/// 339 while_control = WHILE [logical_expression] .
pub fn while_control(input: &str) -> ParseResult<Expression> {
    tuple((tag("WHILE"), logical_expression))
        .map(|(_where, expr)| expr)
        .parse(input)
}

/// 335 until_control = UNTIL [logical_expression] .
pub fn until_control(input: &str) -> ParseResult<Expression> {
    tuple((tag("UNTIL"), logical_expression))
        .map(|(_where, expr)| expr)
        .parse(input)
}

/// 290 return_stmt = RETURN \[ `(` [expression] `)` \] `;` .
pub fn return_stmt(input: &str) -> ParseResult<Statement> {
    tuple((
        tag("RETURN"),
        opt(tuple((char('('), expression, char(')'))).map(|(_open, expr, _close)| expr)),
        char(';'),
    ))
    .map(|(_return, value, _semicolon)| Statement::Return { value })
    .parse(input)
}

/// 308 skip_stmt = SKIP `;` .
pub fn skip_stmt(input: &str) -> ParseResult<Statement> {
    tuple((tag("SKIP"), char(';')))
        .map(|(_skip, _semicolon)| Statement::Skip)
        .parse(input)
}
