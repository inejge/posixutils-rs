#[derive(PartialEq, Debug)]
pub enum ChmodActionOp {
    Add,
    Remove,
    Set,
}

#[derive(Debug)]
pub struct ChmodAction {
    pub op: ChmodActionOp,

    pub copy_user: bool,
    pub copy_group: bool,
    pub copy_others: bool,

    pub read: bool,
    pub write: bool,
    pub execute: bool,
    pub execute_dir: bool,
    pub setuid: bool,
    pub sticky: bool,

    dirty: bool,
}

impl ChmodAction {
    pub fn new() -> ChmodAction {
        ChmodAction {
            op: ChmodActionOp::Set,
            copy_user: false,
            copy_group: false,
            copy_others: false,
            read: false,
            write: false,
            execute: false,
            execute_dir: false,
            setuid: false,
            sticky: false,
            dirty: false,
        }
    }
}

#[derive(Debug)]
pub struct ChmodClause {
    // wholist
    pub user: bool,
    pub group: bool,
    pub others: bool,

    // actionlist
    pub actions: Vec<ChmodAction>,

    dirty: bool,
}

impl ChmodClause {
    pub fn new() -> ChmodClause {
        ChmodClause {
            user: false,
            group: false,
            others: false,
            actions: Vec::new(),
            dirty: false,
        }
    }
}

#[derive(Debug)]
pub struct ChmodSymbolic {
    pub clauses: Vec<ChmodClause>,
}

impl ChmodSymbolic {
    pub fn new() -> ChmodSymbolic {
        ChmodSymbolic {
            clauses: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum ChmodMode {
    Absolute(u32),
    Symbolic(ChmodSymbolic),
}

#[derive(Debug)]
enum ParseState {
    Wholist,
    Actionlist,
    ListOrCopy,
    PermCopy,
    PermList,
    NextClause,
}

pub fn parse(mode: &str) -> Result<ChmodMode, String> {
    match u32::from_str_radix(mode, 8) {
        Ok(m) => {
            return Ok(ChmodMode::Absolute(m));
        }
        Err(_) => {}
    }

    let mut state = ParseState::Wholist;
    let mut done_with_char;
    let mut symbolic = ChmodSymbolic::new();
    let mut clause = ChmodClause::new();
    let mut action = ChmodAction::new();

    for c in mode.chars() {
        done_with_char = false;
        while !done_with_char {
            match state {
                ParseState::Wholist => {
                    done_with_char = true;
                    clause.dirty = true;
                    match c {
                        'u' => clause.user = true,
                        'g' => clause.group = true,
                        'o' => clause.others = true,
                        'a' => {
                            clause.user = true;
                            clause.group = true;
                            clause.others = true;
                        }
                        _ => {
                            state = ParseState::Actionlist;
                            done_with_char = false;
                            clause.dirty = false;
                        }
                    }
                }

                ParseState::Actionlist => {
                    done_with_char = true;
                    state = ParseState::ListOrCopy;
                    action.dirty = true;
                    match c {
                        '+' => action.op = ChmodActionOp::Add,
                        '-' => action.op = ChmodActionOp::Remove,
                        '=' => action.op = ChmodActionOp::Set,
                        _ => {
                            action.dirty = false;
                            done_with_char = false;
                            symbolic.clauses.push(clause);
                            clause = ChmodClause::new();
                            state = ParseState::NextClause;
                        }
                    }
                }

                ParseState::ListOrCopy => match c {
                    'u' | 'g' | 'o' => state = ParseState::PermCopy,
                    _ => state = ParseState::PermList,
                },

                ParseState::PermCopy => {
                    done_with_char = true;
                    match c {
                        'u' => action.copy_user = true,
                        'g' => action.copy_group = true,
                        'o' => action.copy_others = true,
                        _ => {
                            done_with_char = false;
                            clause.actions.push(action);
                            clause.dirty = true;
                            action = ChmodAction::new();
                            state = ParseState::Actionlist;
                        }
                    }
                }

                ParseState::PermList => {
                    done_with_char = true;
                    match c {
                        'r' => action.read = true,
                        'w' => action.write = true,
                        'x' => action.execute = true,
                        'X' => action.execute_dir = true,
                        's' => action.setuid = true,
                        't' => action.sticky = true,
                        _ => {
                            done_with_char = false;
                            clause.actions.push(action);
                            clause.dirty = true;
                            action = ChmodAction::new();
                            state = ParseState::Actionlist;
                        }
                    }
                }

                ParseState::NextClause => {
                    if c != ',' {
                        return Err(format!("unexpected character: {}", c));
                    }
                    done_with_char = true;
                    state = ParseState::Wholist;
                }
            }
        }
    }

    if action.dirty {
        clause.actions.push(action);
        clause.dirty = true;
    }
    if clause.dirty {
        symbolic.clauses.push(clause);
    }

    Ok(ChmodMode::Symbolic(symbolic))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mode() {
        let mode = parse("u=rwX,go=rX").unwrap();
        match mode {
            ChmodMode::Symbolic(s) => {
                assert_eq!(s.clauses.len(), 2);
                let clause = &s.clauses[0];
                assert_eq!(clause.user, true);
                assert_eq!(clause.group, false);
                assert_eq!(clause.others, false);
                assert_eq!(clause.actions.len(), 1);
                let action = &clause.actions[0];
                assert_eq!(action.op, ChmodActionOp::Set);
                assert_eq!(action.copy_user, false);
                assert_eq!(action.copy_group, false);
                assert_eq!(action.copy_others, false);
                assert_eq!(action.read, true);
                assert_eq!(action.write, true);
                assert_eq!(action.execute, false);
                assert_eq!(action.execute_dir, true);
                assert_eq!(action.setuid, false);
                assert_eq!(action.sticky, false);
                let clause = &s.clauses[1];
                assert_eq!(clause.user, false);
                assert_eq!(clause.group, true);
                assert_eq!(clause.others, true);
                assert_eq!(clause.actions.len(), 1);
                let action = &clause.actions[0];
                assert_eq!(action.op, ChmodActionOp::Set);
                assert_eq!(action.copy_user, false);
                assert_eq!(action.copy_group, false);
                assert_eq!(action.copy_others, false);
                assert_eq!(action.read, true);
                assert_eq!(action.write, false);
                assert_eq!(action.execute, false);
                assert_eq!(action.execute_dir, true);
                assert_eq!(action.setuid, false);
                assert_eq!(action.sticky, false);
            }
            _ => panic!("unexpected mode"),
        }
    }
}
