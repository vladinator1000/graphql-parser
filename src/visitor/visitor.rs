use crate::{
    common::{Directive, Text, Type, Value},
    query::{
        Document, Field, FragmentDefinition, FragmentSpread, InlineFragment, OperationDefinition,
        SelectionSet, VariableDefinition,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum QueryAstNode<'ast, T: Text<'ast>> {
    Document(Document<'ast, T>),
    OperationDefinition(OperationDefinition<'ast, T>),
    FragmentDefinition(FragmentDefinition<'ast, T>),
    VariableDefinition(VariableDefinition<'ast, T>),
    SelectionSet(SelectionSet<'ast, T>),
    Field(Field<'ast, T>),
    FragmentSpread(FragmentSpread<'ast, T>),
    InlineFragment(InlineFragment<'ast, T>),
    Directive(Directive<'ast, T>),
}

pub trait QueryVisitor<'ast, T: Text<'ast>> {
    //! By returning different values from the enter and leave functions, the
    //! behavior of the visitor can be altered, including skipping over a sub-tree of
    //! the AST (by returning false), editing the AST by returning a value or null
    //! to remove the value, or to stop the whole traversal by returning BREAK.
    //!
    //! When using visit() to edit an AST, the original AST will not be modified, and
    //! a new version of the AST with the changes applied will be returned from the
    //! visit function.
    fn enter(&mut self, node: &'ast QueryAstNode<'ast, T>) -> VisitorAction<'ast, T> {
        VisitorAction::Skip
    }
    fn leave(&mut self, node: &'ast QueryAstNode<'ast, T>) -> VisitorAction<'ast, T> {
        VisitorAction::Skip
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VisitorAction<'ast, T: Text<'ast>> {
    NoAction,
    Skip,

    /// Stop visiting altogether
    Break,
    DeleteNode,
    ReplaceNode(QueryAstNode<'ast, T>),
}

type Keys<'ast, T: Text<'ast>> = Vec<&'ast QueryAstNode<'ast, T>>;
type KeyNodeTuple<'ast, T: Text<'ast>> = (&'ast QueryAstNode<'ast, T>, &'ast QueryAstNode<'ast, T>);
type Edits<'ast, T: Text<'ast>> = Vec<KeyNodeTuple<'ast, T>>;

struct VisitorStack<'ast, T: Text<'ast>> {
    index: usize,
    keys: Keys<'ast, T>,
    edits: Edits<'ast, T>,
    previous: Box<VisitorStack<'ast, T>>,
}

/// Porting this function from graphql-js 
/// https://github.com/graphql/graphql-js/blob/4493ca3d1281e01635570824f70867aa68610323/src/language/visitor.ts#L247
/// note: We're not supporting the isArray bit to make the port simpler
pub fn visit<'ast, V: QueryVisitor<'ast, T>, T: Text<'ast>>(
    root: &'ast QueryAstNode<'ast, T>,
    visitor: &mut V,
) -> &'ast QueryAstNode<'ast, T> {
    let mut stack: Option<VisitorStack<'ast, T>> = None;
    let mut keys: Keys<'ast, T> = vec![root];
    let mut index: usize = 0;
    let mut edits: Edits<'ast, T> = vec![];
    let mut node: Option<&'ast QueryAstNode<'ast, T>> = None;
    let mut key: Option<&'ast QueryAstNode<'ast, T>> = None;
    let mut parent: Option<&'ast QueryAstNode<'ast, T>> = None;
    let mut path: Vec<&'ast QueryAstNode<'ast, T>> = vec![];
    let mut ancestors: Vec<&'ast QueryAstNode<'ast, T>> = vec![];
    let mut new_root: &'ast QueryAstNode<'ast, T> = root;

    let mut is_first_loop = true;

    loop {
        if is_first_loop {
            is_first_loop = false;
        } else {
            index += 1;
        }

        let is_leaving = index == keys.len();
        let is_edited = is_leaving && edits.len() != 0;

        if is_leaving {
            if ancestors.len() == 0 {
                key = None;
            } else {
                key = Some(path[path.len() - 1]);
            }

            node = parent;
            parent = ancestors.pop();

            if is_edited {
                // This looks like a clone but I'm not sure what its purpose is?
                // Object.defineProperties({}, Object.getOwnPropertyDescriptors(node));

                let edit_offset = 0;
                for (i, edit) in edits.iter().enumerate() {
                    let (key, value) = edit;

                    // It looks like Lee is indexing into the node object here 🤷‍♀️
                    // https://github.com/graphql/graphql-js/blob/4493ca3d1281e01635570824f70867aa68610323/src/language/visitor.ts#L288
                    // node[editKey] = editValue;
                    // TODO: figure out the implications of this on the node type
                }
            }

            if let Some(inner_stack) = stack {
                index = inner_stack.index;
                keys = inner_stack.keys;
                edits = inner_stack.edits;
                stack = Some(*inner_stack.previous);
            }
        } else {
            key = if parent.is_some() {
                Some(keys[index])
            } else {
                None
            };
            // node = parent ? parent[key] : newRoot;
            // Why are we indexing into the parent and what type implications does this have?
            // Is my curent type for parent wrong? Option<&'ast QueryAstNode<'ast, T>>
            // No idea what I'm doing here.

            if node.is_none() {
                continue;
            }

            if parent.is_some() {
                if let Some(key_inner) = key {
                    path.push(key_inner);
                }
            }
        }

        if stack.is_none() {
            break;
        }
    }

    if edits.len() != 0 {
        let (_key, value) = edits[edits.len() - 1];
        new_root = value;
    }

    return new_root;
}

#[cfg(test)]
mod visitor_tests {
    use k9::assert_equal;

    use super::*;
    use crate::parse_query;
    use crate::query::*;
    use crate::Pos;

    #[test]
    fn node_editing_on_enter() {
        let ast: Document<&str> =
            parse_query("{ a, b, c { a, b, c } }").expect("Failed to parse query");

        struct SelectionSetVisitor<'ast, T: Text<'ast>> {
            selection_sets: Vec<&'ast SelectionSet<'ast, T>>,
        }

        impl<'ast, T: Text<'ast>> QueryVisitor<'ast, T> for SelectionSetVisitor<'ast, T> {
            fn enter(&mut self, node: &'ast QueryAstNode<'ast, T>) -> VisitorAction<'ast, T> {
                match node {
                    QueryAstNode::OperationDefinition(definition) => match definition {
                        OperationDefinition::SelectionSet(selection_set) => {
                            self.selection_sets.push(selection_set);

                            let empty_position = Pos { line: 0, column: 0 };

                            let new_field = Field {
                                position: empty_position,
                                alias: None,
                                name: "enter".into(),
                                arguments: vec![],
                                directives: vec![],
                                selection_set: SelectionSet {
                                    span: (empty_position, empty_position),
                                    items: vec![],
                                },
                            };

                            let new_selection_set = QueryAstNode::SelectionSet(SelectionSet {
                                span: (empty_position, empty_position),
                                items: vec![Selection::Field(new_field)],
                            });

                            VisitorAction::ReplaceNode(new_selection_set)
                        }
                        _ => VisitorAction::NoAction,
                    },
                    _ => VisitorAction::NoAction,
                }
            }

            fn leave(&mut self, node: &'ast QueryAstNode<'ast, T>) -> VisitorAction<'ast, T> {
                VisitorAction::NoAction
            }
        }

        let mut selection_set_visitor = SelectionSetVisitor {
            selection_sets: Vec::new(),
        };

        if let QueryAstNode::Document(edited_document) =
            visit(&QueryAstNode::Document(ast), &mut selection_set_visitor)
        {
            let edited = edited_document.to_string();
            let expected = "{ a, b, c { enter, enter, enter } }".to_string();
            assert_equal!(edited, expected);
        }
    }

    #[test]
    fn node_editing_on_leave() {}
}
