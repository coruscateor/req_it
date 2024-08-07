
use std::sync::{atomic::{AtomicU64, Ordering}, Arc};

#[derive(Clone)]
pub struct Incrementor
{

    count: Arc<AtomicU64>

}

impl Incrementor
{

    pub fn new(count: Arc<AtomicU64>) -> Self
    {

        Self
        {

            count

        }

    }

    pub fn inc(&self)
    {

        self.count.fetch_add(1, Ordering::SeqCst);

    }

    pub fn count(&self) -> u64
    {

        self.count.load(Ordering::Acquire)//Ordering::SeqCst) //Ordering::Acquire)

    }

    pub fn has_messages(&self) -> bool
    {

        self.count() > 0

    }

    pub fn is_relevant(&self) -> bool
    {

        Arc::strong_count(&self.count) > 1

    }

}

#[derive(Clone)]
pub struct Decrementor
{

    count: Arc<AtomicU64>

}

impl Decrementor
{

    pub fn new(count: Arc<AtomicU64>) -> Self
    {

        Self
        {

            count

        }

    }

    pub fn dec(&self)
    {

        self.count.fetch_sub(1, Ordering::SeqCst);

    }

    pub fn count(&self) -> u64
    {

        self.count.load(Ordering::Acquire) //Ordering::SeqCst) //Ordering::Acquire)

    }
    
    pub fn has_messages(&self) -> bool
    {

        self.count() > 0

    }

    pub fn is_relevant(&self) -> bool
    {

        Arc::strong_count(&self.count) > 1

    }

}

pub fn inc_dec() -> (Incrementor, Decrementor)
{

    let count = Arc::new(AtomicU64::new(0));

    (Incrementor::new(count.clone()), Decrementor::new(count))

}
