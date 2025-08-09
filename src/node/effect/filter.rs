use crate::{
    AssetLoader, Error, Event, GraphNode, IirFilter, Message, Node,
    abstraction::{ChildConfig, NodeConfig, defaults},
    consts,
};
use biquad::{Biquad, Coefficients, DirectForm1, Type, frequency::*};
use serde::Deserialize;

const SAMPLING_FREQUENCY_KHZ: f32 = 24.0;

#[derive(Deserialize, Clone)]
pub struct Filter {
    #[serde(default = "defaults::none_id")]
    pub node_id: Option<u64>,
    pub filter: Option<(IirFilter, f32)>,
    pub source: ChildConfig,
}

impl NodeConfig for Filter {
    fn to_node(&self, asset_loader: &mut dyn AssetLoader) -> Result<GraphNode, Error> {
        let source = self.source.0.to_node(asset_loader)?;
        Ok(Box::new(FilterNode::new(
            self.node_id,
            self.filter,
            source,
        )?))
    }

    fn clone_child_configs(&self) -> Option<Vec<ChildConfig>> {
        Some(vec![self.source.clone()])
    }

    fn asset_source(&self) -> Option<&str> {
        None
    }

    fn duplicate(&self) -> Box<dyn NodeConfig + Send + Sync + 'static> {
        Box::new(self.clone())
    }
}

pub struct FilterNode {
    node_id: u64,
    filter: Option<IirFilter>,
    consumer: GraphNode,
    intermediate_buffer: Vec<f32>,
    base_frequency: f32,
    left_filter: DirectForm1<f32>,
    right_filter: DirectForm1<f32>,
}

impl FilterNode {
    pub fn new(
        node_id: Option<u64>,
        filter: Option<(IirFilter, f32)>,
        consumer: GraphNode,
    ) -> Result<Self, Error> {
        let (filter, base_frequency) = match filter {
            Some((filter, cutoff_frequency)) => (Some(filter), cutoff_frequency),
            None => (None, 1000.0),
        };
        let coefficients =
            Self::coefficients_for_filter(filter.unwrap_or(IirFilter::LowPass), base_frequency)?;
        Ok(Self {
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            filter,
            consumer,
            intermediate_buffer: vec![0.0; consts::BUFFER_SIZE * consts::CHANNEL_COUNT],
            base_frequency,
            left_filter: DirectForm1::new(coefficients.clone()),
            right_filter: DirectForm1::new(coefficients),
        })
    }

    fn coefficients_for_filter(
        filter: IirFilter,
        cutoff_frequency: f32,
    ) -> Result<Coefficients<f32>, Error> {
        let filter_type: Type<f32> = match filter {
            IirFilter::SinglePoleLowPassApprox => Type::SinglePoleLowPassApprox,
            IirFilter::SinglePoleLowPass => Type::SinglePoleLowPass,
            IirFilter::LowPass => Type::LowPass,
            IirFilter::HighPass => Type::HighPass,
            IirFilter::BandPass => Type::BandPass,
            IirFilter::Notch => Type::Notch,
            IirFilter::AllPass => Type::AllPass,
            IirFilter::LowShelf { db_gain } => Type::LowShelf(db_gain),
            IirFilter::HighShelf { db_gain } => Type::HighShelf(db_gain),
            IirFilter::PeakingEQ { db_gain } => Type::PeakingEQ(db_gain),
        };
        let coefficients = Coefficients::from_params(
            filter_type,
            SAMPLING_FREQUENCY_KHZ.khz(),
            cutoff_frequency.hz(),
            biquad::Q_BUTTERWORTH_F32,
        )?;
        Ok(coefficients)
    }

    fn set_filter(
        &mut self,
        filter: Option<IirFilter>,
        cutoff_frequency: f32,
    ) -> Result<(), Error> {
        self.filter = filter.clone();
        self.base_frequency = cutoff_frequency;
        if let Some(filter) = filter {
            let coefficients = Self::coefficients_for_filter(filter, cutoff_frequency)?;
            self.left_filter.update_coefficients(coefficients.clone());
            self.right_filter.update_coefficients(coefficients);
        }
        Ok(())
    }

    fn set_frequency_shift(&mut self, frequency_shift: f32) -> Result<(), Error> {
        let Some(filter) = self.filter else {
            return Ok(());
        };
        self.set_filter(Some(filter), self.base_frequency + frequency_shift)
    }
}

impl Node for FilterNode {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn set_node_id(&mut self, node_id: u64) {
        self.node_id = node_id;
    }

    fn duplicate(&self) -> Result<GraphNode, Error> {
        let transition = Self {
            node_id: self.node_id,
            filter: self.filter.clone(),
            consumer: self.consumer.duplicate()?,
            intermediate_buffer: vec![0.0; consts::BUFFER_SIZE * consts::CHANNEL_COUNT],
            base_frequency: self.base_frequency,
            left_filter: self.left_filter.clone(),
            right_filter: self.right_filter.clone(),
        };
        Ok(Box::new(transition))
    }

    fn try_consume_event(&mut self, event: &Message) -> bool {
        match event.data {
            Event::Filter {
                filter,
                cutoff_frequency,
            } => {
                match self.set_filter(Some(filter), cutoff_frequency) {
                    Ok(()) => {
                        self.base_frequency = cutoff_frequency;
                    }
                    Err(error) => println!("Error setting filter: {:?}", error),
                };
                true
            }
            Event::FilterFrequencyShift(shift) => {
                if let Err(error) = self.set_frequency_shift(shift) {
                    println!("Error setting filter: {:?}", error);
                };
                true
            }
            _ => false,
        }
    }

    fn propagate(&mut self, event: &Message) {
        self.consumer.on_event(event);
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        if !self.filter.is_some() {
            self.consumer.fill_buffer(buffer);
            return;
        }
        let buffer_size = buffer.len();
        let sample_count = buffer_size / consts::CHANNEL_COUNT;
        self.intermediate_buffer.fill(0.0);
        self.consumer.fill_buffer(&mut self.intermediate_buffer);
        for i in 0..sample_count {
            let index = i * 2;
            buffer[index] += self.left_filter.run(self.intermediate_buffer[index]);
            buffer[index + 1] += self.right_filter.run(self.intermediate_buffer[index + 1]);
        }
    }

    fn replace_children(&mut self, children: &[GraphNode]) -> Result<(), Error> {
        if children.len() != 1 {
            return Err(Error::User(
                "TransitionEnvelope requires one child".to_owned(),
            ));
        }
        self.consumer = children[0].duplicate()?;
        Ok(())
    }
}
