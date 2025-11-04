use bevy::prelude::*;
use bevy_common_systems::modding::prelude::*;

use super::world::NovaEventWorld;

pub mod prelude {
    pub use super::{
        DebugMessageActionConfig, EventActionConfig, ObjectiveActionConfig, VariableError,
        VariableExpressionNode, VariableFactorNode, VariableLiteral, VariableSetActionConfig,
        VariableTermNode,
    };
}

#[derive(Clone, Debug)]
pub enum EventActionConfig {
    DebugMessage(DebugMessageActionConfig),
    VariableSet(VariableSetActionConfig),
    Objective(ObjectiveActionConfig),
}

impl EventAction<NovaEventWorld> for EventActionConfig {
    fn action(&self, world: &mut NovaEventWorld, info: &GameEventInfo) {
        match self {
            EventActionConfig::DebugMessage(config) => {
                config.action(world, info);
            }
            EventActionConfig::VariableSet(config) => {
                config.action(world, info);
            }
            EventActionConfig::Objective(config) => {
                config.action(world, info);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum VariableError {
    UndefinedVariable(String),
    TypeMismatch(String),
    DivisionByZero,
}

#[derive(Clone, Debug)]
pub enum VariableLiteral {
    String(String),
    Number(f64),
    Boolean(bool),
}

#[derive(Clone, Debug)]
pub enum VariableFactorNode {
    Parens(Box<VariableExpressionNode>),
    Literal(VariableLiteral),
    Name(String),
}

impl VariableFactorNode {
    pub fn new_literal(lit: VariableLiteral) -> Self {
        VariableFactorNode::Literal(lit)
    }

    pub fn new_name<S: Into<String>>(name: S) -> Self {
        VariableFactorNode::Name(name.into())
    }

    pub fn new_parens(expr: VariableExpressionNode) -> Self {
        VariableFactorNode::Parens(Box::new(expr))
    }

    fn evaluate(&self, world: &NovaEventWorld) -> Result<VariableLiteral, VariableError> {
        match self {
            VariableFactorNode::Parens(expr) => expr.evaluate(world),
            VariableFactorNode::Literal(lit) => Ok(lit.clone()),
            VariableFactorNode::Name(name) => world
                .variables
                .get(name)
                .cloned()
                .ok_or_else(|| VariableError::UndefinedVariable(name.clone())),
        }
    }
}

#[derive(Clone, Debug)]
pub enum VariableTermNode {
    Multiply(Box<VariableFactorNode>, Box<VariableTermNode>),
    Divide(Box<VariableFactorNode>, Box<VariableTermNode>),
    Factor(VariableFactorNode),
}

impl VariableTermNode {
    pub fn new_multiply(left: VariableFactorNode, right: VariableTermNode) -> Self {
        VariableTermNode::Multiply(Box::new(left), Box::new(right))
    }

    pub fn new_divide(left: VariableFactorNode, right: VariableTermNode) -> Self {
        VariableTermNode::Divide(Box::new(left), Box::new(right))
    }

    pub fn new_factor(factor: VariableFactorNode) -> Self {
        VariableTermNode::Factor(factor)
    }

    fn evaluate(&self, world: &NovaEventWorld) -> Result<VariableLiteral, VariableError> {
        match self {
            VariableTermNode::Multiply(left, right) => {
                let left_val = left.evaluate(world)?;
                let right_val = right.evaluate(world)?;
                match (left_val, right_val) {
                    (VariableLiteral::Number(l), VariableLiteral::Number(r)) => {
                        Ok(VariableLiteral::Number(l * r))
                    }
                    (VariableLiteral::Boolean(l), VariableLiteral::Boolean(r)) => {
                        Ok(VariableLiteral::Boolean(l && r))
                    }
                    _ => Err(VariableError::TypeMismatch(
                        "Expected numbers for multiplication".to_string(),
                    )),
                }
            }
            VariableTermNode::Divide(left, right) => {
                let left_val = left.evaluate(world)?;
                let right_val = right.evaluate(world)?;
                match (left_val, right_val) {
                    (VariableLiteral::Number(l), VariableLiteral::Number(r)) => {
                        if r == 0.0 {
                            Err(VariableError::DivisionByZero)
                        } else {
                            Ok(VariableLiteral::Number(l / r))
                        }
                    }
                    _ => Err(VariableError::TypeMismatch(
                        "Expected numbers for division".to_string(),
                    )),
                }
            }
            VariableTermNode::Factor(factor) => factor.evaluate(world),
        }
    }
}

#[derive(Clone, Debug)]
pub enum VariableExpressionNode {
    Add(Box<VariableTermNode>, Box<VariableExpressionNode>),
    Subtract(Box<VariableTermNode>, Box<VariableExpressionNode>),
    Term(VariableTermNode),
}

impl VariableExpressionNode {
    pub fn new_add(left: VariableTermNode, right: VariableExpressionNode) -> Self {
        VariableExpressionNode::Add(Box::new(left), Box::new(right))
    }

    pub fn new_subtract(left: VariableTermNode, right: VariableExpressionNode) -> Self {
        VariableExpressionNode::Subtract(Box::new(left), Box::new(right))
    }

    pub fn new_term(term: VariableTermNode) -> Self {
        VariableExpressionNode::Term(term)
    }

    fn evaluate(&self, world: &NovaEventWorld) -> Result<VariableLiteral, VariableError> {
        match self {
            VariableExpressionNode::Add(left, right) => {
                let left_val = left.evaluate(world)?;
                let right_val = right.evaluate(world)?;
                match (left_val, right_val) {
                    (VariableLiteral::Number(l), VariableLiteral::Number(r)) => {
                        Ok(VariableLiteral::Number(l + r))
                    }
                    (VariableLiteral::Boolean(l), VariableLiteral::Boolean(r)) => {
                        Ok(VariableLiteral::Boolean(l || r))
                    }
                    (VariableLiteral::String(l), VariableLiteral::String(r)) => {
                        Ok(VariableLiteral::String(l + &r))
                    }
                    _ => Err(VariableError::TypeMismatch(
                        "Expected numbers for addition".to_string(),
                    )),
                }
            }
            VariableExpressionNode::Subtract(left, right) => {
                let left_val = left.evaluate(world)?;
                let right_val = right.evaluate(world)?;
                match (left_val, right_val) {
                    (VariableLiteral::Number(l), VariableLiteral::Number(r)) => {
                        Ok(VariableLiteral::Number(l - r))
                    }
                    _ => Err(VariableError::TypeMismatch(
                        "Expected numbers for subtraction".to_string(),
                    )),
                }
            }
            VariableExpressionNode::Term(term) => term.evaluate(world),
        }
    }
}

#[derive(Clone, Debug)]
pub struct VariableSetActionConfig {
    pub key: String,
    pub expression: VariableExpressionNode,
}

impl EventAction<NovaEventWorld> for VariableSetActionConfig {
    fn action(&self, world: &mut NovaEventWorld, _: &GameEventInfo) {
        match self.expression.evaluate(world) {
            Ok(literal) => {
                world.variables.insert(self.key.clone(), literal);
            }
            Err(e) => {
                error!(
                    "VariableSetActionConfig: failed to evaluate expression for key '{}': {:?}",
                    self.key, e
                );
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct DebugMessageActionConfig {
    pub message: String,
}

impl EventAction<NovaEventWorld> for DebugMessageActionConfig {
    fn action(&self, _: &mut NovaEventWorld, _: &GameEventInfo) {
        debug!("Event Action Message: {}", self.message);
    }
}

// TODO: Think more about how to do objectives

#[derive(Clone, Debug)]
pub enum ObjectiveCondition {
    Win,
    Lose,
}

#[derive(Clone, Debug)]
pub struct ObjectiveProgress {
    pub current: u32,
    pub target: u32,
}

#[derive(Clone, Debug)]
pub struct ObjectiveActionConfig {
    pub id: String,
    pub message: String,
    pub condition: ObjectiveCondition,
}

impl ObjectiveActionConfig {
    pub fn new(id: &str, message: &str) -> Self {
        Self {
            id: id.to_string(),
            message: message.to_string(),
            condition: ObjectiveCondition::Win,
        }
    }

    pub fn win(mut self) -> Self {
        self.condition = ObjectiveCondition::Win;
        self
    }

    pub fn lose(mut self) -> Self {
        self.condition = ObjectiveCondition::Lose;
        self
    }
}

impl EventAction<NovaEventWorld> for ObjectiveActionConfig {
    fn action(&self, world: &mut NovaEventWorld, _: &GameEventInfo) {
        world.objectives.push(self.clone());
    }
}
