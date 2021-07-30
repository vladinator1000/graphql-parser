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
    index: i32,
    keys: Keys<'ast, T>,
    edits: Edits<'ast, T>,
    previous: Box<VisitorStack<'ast, T>> 
}


/// Porting this function from graphql-js
/// https://github.com/graphql/graphql-js/blob/4493ca3d1281e01635570824f70867aa68610323/src/language/visitor.ts#L247
pub fn visit<'ast, V: QueryVisitor<'ast, T>, T: Text<'ast>>(
    root: &'ast QueryAstNode<'ast, T>,
    visitor: &mut V,
) -> QueryAstNode<'ast, T> {
    let mut stack: Option<VisitorStack<'ast, T>> = None;
    let mut keys: Keys<'ast, T> = vec![root];
    let mut index = -1;
    let edits: Edits<'ast, T> = vec![];
    let mut node: Option<&'ast QueryAstNode<'ast, T>> = None;
    let mut key: Option<&'ast QueryAstNode<'ast, T>> = None;
    let mut parent: Option<&'ast QueryAstNode<'ast, T>> = None;
    let path: Vec<&'ast QueryAstNode<'ast, T>> = vec![];
    let ancestors: Vec<&'ast QueryAstNode<'ast, T>> = vec![];
    let mut new_root: &'ast QueryAstNode<'ast, T> = root;

    unimplemented!()
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
                                arguments:  vec![],
                                directives: vec![],
                                selection_set: SelectionSet {
                                    span: (empty_position, empty_position),
                                    items: vec![]
                                }
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
            selection_sets: Vec::new()
        };

        if let QueryAstNode::Document(edited_document) = visit(&QueryAstNode::Document(ast), &mut selection_set_visitor) {
            let edited = edited_document.to_string();
            let expected = "{ a, b, c { enter, enter, enter } }".to_string();
            assert_equal!(edited, expected);
        }
    }
    

    #[test]
    fn node_editing_on_leave() {}
}
