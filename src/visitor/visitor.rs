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

pub fn visit<'ast, V: QueryVisitor<'ast, T>, T: Text<'ast>>(
    node: &'ast QueryAstNode<'ast, T>,
    visitor: &mut V,
) -> QueryAstNode<'ast, T> {
    unimplemented!()
}

#[cfg(test)]
mod visitor_tests {
    use super::*;
    use crate::parse_query;
    use crate::query::*;
    use crate::Pos;

    #[test]
    fn allows_editing_a_node_both_on_enter_and_on_leave() {
        let ast: Document<&str> =
            parse_query("{ a, b, c { a, b, c } }").expect("Failed to parse query");

        struct SelectionSetVisitor<'ast, T: Text<'ast>> {
            selection_set: &'ast SelectionSet<'ast, T>,
        }

        impl<'ast, T: Text<'ast>> QueryVisitor<'ast, T> for SelectionSetVisitor<'ast, T> {
            fn enter(&mut self, node: &'ast QueryAstNode<'ast, T>) -> VisitorAction<'ast, T> {
                match node {
                    QueryAstNode::OperationDefinition(definition) => {
                       match definition {
                           OperationDefinition::SelectionSet(selection_set) => {
                               self.selection_set = selection_set;
                               VisitorAction::ReplaceNode(QueryAstNode::SelectionSet(SelectionSet {
                                   span: (Pos { line: 0, column: 0 }, Pos { line: 0, column: 0 }),
                                   items: vec![]
                               }))
                           },
                           _ => VisitorAction::NoAction
                       }
                    },
                    _ => VisitorAction::NoAction
                }
            }
        }
    }
}
