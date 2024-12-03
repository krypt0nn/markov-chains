use tinyvec::TinyVec;

#[derive(Default, Debug, Clone)]
pub struct Classifier {
    layers: TinyVec<[ClassifierLayer; 4]>
}

impl Classifier {
    /// Create classifier with random layers with given sizes.
    pub fn random(layers: &[usize], outputs: usize) -> Self {
        let mut final_layers = TinyVec::<[ClassifierLayer; 4]>::new();

        for i in 0..layers.len() - 1 {
            final_layers.push(ClassifierLayer::random(layers[i], layers[i + 1]));
        }

        final_layers.push(ClassifierLayer::random(layers[layers.len() - 1], outputs));

        Self {
            layers: final_layers
        }
    }

    #[inline]
    pub fn input_len(&self) -> usize {
        self.layers[0].input_len()
    }

    #[inline]
    pub fn output_len(&self) -> usize {
        self.layers[self.layers.len() - 1].output_len()
    }
}

#[derive(Default, Debug, Clone)]
/// Layer of the message classifier.
pub struct ClassifierLayer {
    nodes: TinyVec<[TinyVec<[f32; 64]>; 64]>
}

impl ClassifierLayer {
    /// Create new layer with random weights, input and output nodes.
    pub fn random(inputs: usize, outputs: usize) -> Self {
        let mut nodes = TinyVec::<[TinyVec::<[f32; 64]>; 64]>::with_capacity(inputs);

        for _ in 0..inputs {
            let mut weights = TinyVec::<[f32; 64]>::with_capacity(outputs);

            for _ in 0..outputs {
                weights.push(fastrand::f32());
            }

            nodes.push(weights);
        }

        Self {
            nodes
        }
    }

    #[inline]
    pub fn input_len(&self) -> usize {
        self.nodes.len()
    }

    #[inline]
    pub fn output_len(&self) -> usize {
        self.nodes[0].len()
    }

    /// Calculate outputs of the layer from the given inputs.
    pub fn propagate(&self, inputs: TinyVec<[f32; 64]>) -> TinyVec<[f32; 64]> {
        let outputs_len = self.output_len();

        let mut outputs = TinyVec::<[f32; 64]>::with_capacity(outputs_len);

        for _ in 0..outputs_len {
            outputs.push(0.0);
        }

        for (i, weights) in self.nodes.iter().enumerate() {
            for (j, weight) in weights.iter().enumerate() {
                outputs[j] += inputs[i] * *weight;
            }
        }

        for output in &mut outputs {
            *output /= outputs_len as f32;
        }

        outputs
    }
}
