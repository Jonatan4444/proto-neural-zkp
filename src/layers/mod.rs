use std::fmt::{Display, Formatter, Result};

use ndarray::{ArcArray, ArrayD, ArrayViewD, Ix1, Ix2, Ix3};
use serde::{Deserialize, Serialize};

pub mod conv;
pub mod flatten;
pub mod fully_connected;
pub mod maxpool;
pub mod normalize;
pub mod relu;

pub trait Layer: Into<LayerJson> {
    #[must_use]
    fn apply(&self, input: &ArrayViewD<f32>) -> ArrayD<f32>;

    fn input_shape(&self) -> Vec<usize>;

    #[must_use]
    fn name(&self) -> &str;

    #[must_use]
    fn num_params(&self) -> usize;

    #[must_use]
    fn num_muls(&self) -> usize;

    fn output_shape(&self) -> Vec<usize>;
}

impl Display for Box<dyn Layer> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{:<20} | {:?}{:<5} | {:<5} | {:<5}",
            self.name(),
            self.output_shape(),
            "",
            self.num_params(),
            self.num_muls(),
        )
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layers {
    Convolution,
    MaxPool,
    Relu,
    Flatten,
    FullyConnected,
    Normalize,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum LayerJson {
    Convolution {
        kernel: ArcArray<f32, Ix3>,
    },
    MaxPool {
        window: usize,
    },
    FullyConnected {
        weights: ArcArray<f32, Ix2>,
        biases:  ArcArray<f32, Ix1>,
    },
    Relu,
    Flatten,
    Normalize,
}

// Into for each layer
impl Into<LayerJson> for conv::Convolution {
    fn into(self) -> LayerJson {
        LayerJson::Convolution {
            kernel: self.kernel.clone(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct NNJson {
    pub layers: Vec<Layers>,
}

impl TryFrom<LayerJson> for Box<dyn Layer> {
    type Error = ();

    fn try_from(value: LayerJson) -> std::result::Result<Self, ()> {
        Ok(match value {
            LayerJson::Convolution { kernel } => Box::new(conv::Convolution::new(kernel)),
            LayerJson::MaxPool { window } => Box::new(maxpool::MaxPool::new(window)),
            LayerJson::FullyConnected { weights, biases } => {
                Box::new(fully_connected::FullyConnected::new())
            }
            LayerJson::Flatten => Box::new(flatten::Flatten::new()),
            LayerJson::Relu => Box::new(relu::Relu::new()),
            LayerJson::Normalize => Box::new(normalize::Normalize::new()),
        })
    }
}

impl From<NeuralNetwork> for NNJson {
    fn from(nn: NeuralNetwork) -> Self {
        Self {
            layers: nn.layers.into_iter().map(|l| l.into()).collect(),
        }
    }
}

impl TryFrom<NNJson> for NeuralNetwork {
    type Error = ();

    fn try_from(value: NNJson) -> std::result::Result<Self, ()> {
        Ok(Self {
            layers: value
                .layers
                .into_iter()
                .map(|l| l.try_into())
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

pub struct JSONLayer {
    layer_type:  Layers,
    kernel:      Option<ArcArray<f32, Ix3>>,
    kernel_size: Option<i8>,
    weights:     Option<ArcArray<f32, Ix2>>,
    biases:      Option<ArcArray<f32, Ix1>>,
}

#[derive(Serialize, Deserialize)]
#[serde(into = "NNJson", try_from = "NNJson")]
pub struct NeuralNetwork {
    layers: Vec<Box<dyn Layer>>,
}

impl NeuralNetwork {
    pub fn new() -> Self {
        Self { layers: vec![] }
    }

    pub fn add_layer(&mut self, layer: Box<dyn Layer>) {
        self.layers.push(layer);
    }

    pub fn apply(&self, input: &ArrayViewD<f32>, dim: usize) -> Option<ArrayD<f32>> {
        if dim == 3 {
            let mut output = input.view().into_owned();

            for layer in &self.layers {
                // TODO: add dimensionality sanity checks
                output = layer.apply(&output.view());
                println!("{}", layer);
            }
            Some(output)
        } else {
            None
        }
    }
}

impl Default for NeuralNetwork {
    fn default() -> Self {
        Self::new()
    }
}
