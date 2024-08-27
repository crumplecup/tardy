use crate::{Act, Arrive, Frame, FRAMES};
use convert_case::Casing;
use std::time::Duration;
use std::{fs, path};
use tokio::sync::{mpsc, oneshot};
use tokio::time;
use winit::event_loop;

/// The purpose of the `Imp` struct is to perform application actions without the user's consent.
///
/// The `Imp` will perform its actions from a separate task.  For this reason, I am reluctant to
/// pass any mutable references to [`crate::App`].  First, I do not want to have to synchronize access to
/// `App` with some sort of atomics or mutex.  That seems unnecessary for some light hijinks.
/// Second, I do not want to run into any issues taking multiple mutable references to `App`.
///
/// Instead, we will make the `App` struct complicit in the hijinks of the `Imp` using message
/// passing.  The `Imp` will hold the transmitter and the `App` will hold the receiver.
#[derive(Debug, derive_new::new, derive_getters::Getters)]
pub struct Imp {
    /// Holds frames for new windows.
    frames: Vec<Frame>,
    /// Name of imp, not guaranteed to be unique.
    name: String,
    /// Inspirational quotes used to spam the console.
    quotes: Quotes,
    /// Send hijinks to the Imp King.
    tx: mpsc::Sender<Hijinks>,
}

impl Imp {
    /// The `pause` method calls [`time::sleep`] from the [`tokio`] crate.  This seems to be the
    /// de-facto way to demonstrate asynchronicity when otherwise the operation would complete too
    /// quickly.  In this case, we want the occassional interuption of the user's workflow to be
    /// funny, so it needs to be intermittent enough to be considered at most a mild annoyance.
    /// The mild annoyance in real life is how long geospatial operations take, especially in a
    /// network context.
    ///
    /// This method calls [`rand::random`] to obtain a `u16` value.  The maximum value of 65,535
    /// millis is just over a minute, which is reasonable for our use case.  There is no threshold
    /// on the minimum, which can result in several quick successive actions from a process.
    #[tracing::instrument]
    pub async fn pause() {
        let pause: u16 = rand::random();
        tracing::info!("Pausing for {pause} millis");
        time::sleep(Duration::from_millis(pause as u64)).await;
    }

    /// The `instigate` method prompts the application to create a new window.  The purpose of this
    /// method is to amuse or annoy the user by creating a window when they are not too busy.
    ///
    /// The method calls [`std::vec::Vec::pop`] of the `frames` field. When a frame is present, we create
    /// a [`Meddle`], specifying the [`Act::NewWindow`] as the variant, and including the imp's
    /// name in the window title.  When a frame is not present, this function calls for more
    /// frames, and waits to receive them before exiting.  If no frames are forthcoming this call
    /// may lock up.
    #[tracing::instrument(skip_all)]
    pub async fn instigate(&mut self) -> Arrive<()> {
        let frame = self.frames.pop();
        if frame.is_some() {
            let meddle = Meddle::new(Act::NewWindow, frame, format!("{}'s Window", self.name()));
            tracing::info!("Hijinks instigated.");
            self.tx.send(Hijinks::Meddle(meddle)).await?;
        } else {
            tracing::warn!("{} is out of frames.", self.name);
            self.filch().await?;
        }
        Ok(())
    }

    /// The `filch` method absconds with [`Frame`] by asking the [`crate::App`] for them and then
    /// waiting for a response.  Hard to really call this stealing, but imps have their pride.
    ///
    /// This method uses an element from Alice Rhyl's [blog]("https://ryhl.io/blog/actors-with-tokio/").
    /// While I have read this post several times over the years, it never clicked with me, any more
    /// than the other posts and tutorials I had found.  I think the issues stemmed from a poor
    /// understand of why, when and where to use threads.  Partially, my past experience had not
    /// exposed me to them, and my use cases had not called for them, until one day they did...
    ///
    /// I often imagine a comic where a programmer ponders what arcane data type to use in order to
    /// model a new amazing idea.  After listing all the fancy buzzwords and acronyms (think HRTB
    /// and RPIT), the perfect tool is a struct.  And maybe an enum.  Surprisingly, this often is
    /// all I need, especially if I am leaning on the standard library or a third-party crate to do
    /// the heavy lifting.
    ///
    /// After a few years went by, I ran into a code duplication issue that mystified me.  I had
    /// many similar sources that I was reading into a common data type so they could share a set
    /// of methods.  One day a lightbulb went on and I realized that I needed a trait.  So *thats*
    /// what they were for.  I was doing so much work with pipeline style workflows, that I had not run into
    /// the use case until that moment.
    ///
    /// When I finally ran into a use case for threads, my initial reaction was that I "needed
    /// async".  Never mind that I did not properly understand Chapter 16 on threads.  In fact, the
    /// primary use case I had in mind was for keeping long-running tasks from locking up the UI on
    /// a user application.  After describing myself as a "back-end developer" for years as a
    /// justification of why my code never seemed to actually do anything, I am finally exploring
    /// the apex of front-end development, the GUI interface.
    ///
    /// The reason my programs run slow may have nothing to do with geospatial anything, maybe I am
    /// just writing O^N! algorithms everywhere.  Either way, a quick fix is unlikely, so in the
    /// interim I would like to learn to use threads to process long-running tasks in the
    /// background while the user can continue to interact with the application.
    ///
    /// Reading Chapter 17 helped me to identify key components that were thematic to async
    /// generally, such as the message-passing pattern used in `tardy`.  With these basic building
    /// blocks as a reference, I was able to see Alice's blog with new eyes, in that I could
    /// distinguish parts of the code which adhered to the expected pattern from odd bits that
    /// diverged.
    ///
    /// The part that jumped out to me as unique and helpful works like a self-addressed stamped
    /// envelope (SASE).  The actor sends a message to the handler, and the message contains a sender for
    /// return replies.  In my case, imps are sending requests for frames into the application.
    /// The problems I was wrestling with:
    ///
    ///  * How do I get the frames back?  
    ///  * Does every imp need to include an ID?
    ///  * Do I need a hashmap of ID's and imps somewhere?
    ///  * Do I need to store tx and rx's in all these structs?
    ///
    ///  The SASE pattern allows me to avoid the big-government approach entirely.  The channel is
    ///  local to the function.  It's probably less efficient than storing a receiver and reusing
    ///  it, but imps are lazy so they do not mind.  The imp sends a [`Hijinks::Filch`] variant
    ///  that contains a [`Filch`] struct, with a single field called `tx` containing a
    ///  [`oneshot::Sender`].  The app uses the enclosed transmitter to send a vector of [`Frame`]
    ///  instances back to the requestor.  We then await the receiver.
    ///
    ///  Currently there is no timeout mechanism, so if the app does not respond this process is
    ///  likely to hang.
    #[tracing::instrument(skip_all)]
    pub async fn filch(&mut self) -> Arrive<()> {
        let (tx, rx) = oneshot::channel();
        let filch = Filch::new(tx);
        let hijinks = Hijinks::Filch(filch);
        tracing::info!("{} is trash talking.", self.name());
        let _ = self.tx.send(hijinks).await;
        let frames = rx.await?;
        tracing::info!("{} stole frames.", self.name());
        self.frames = frames;
        Ok(())
    }

    /// The `spoil` method closes an open window at random, whether you like it or not.
    /// The purpose of this method is to create pain, woe, despair, or mild annoyance by closing a
    /// randomly selected open window.  Unfortunately, open windows currently do not do anything,
    /// so it's hard to get too worked up about it.
    #[tracing::instrument(skip_all)]
    pub async fn spoil(&mut self) -> Arrive<()> {
        let meddle = Meddle::new(Act::CloseWindow, None, self.name().clone());
        tracing::info!("Spoiler alert.");
        self.tx.send(Hijinks::Meddle(meddle)).await?;
        Ok(())
    }

    /// The `meddle` method is how an `Imp` signals intent to take an action.
    /// The purpose of this method is randomize the choice of action.
    /// Current choices include opening a window using [`Imp::instigate`] and closing a window
    /// using [`Imp::spoil`].  We use the [`rand::random`] method to produce a coin flip between
    /// the two.  Upsetting this balance could starve the application of new windows or glut the
    /// user with an abundance of them.
    #[tracing::instrument(skip_all)]
    pub async fn meddle(&mut self) -> Arrive<()> {
        if rand::random() {
            self.instigate().await?;
        } else {
            self.spoil().await?;
        }
        Self::pause().await;
        Ok(())
    }

    /// The `vandalize` method logs an inspirational quote to the console at the `INFO` level.
    /// The purpose of this method is to spam the logs with distracting and uplifting quotes,
    /// because who doesn't find verbose logs annoying?
    ///
    /// I am frustrated by the lack of support for [`rand::Rng::gen_range`] in an async context.
    /// Surely there is an anologues tool I can reach for here.  Instead, I randomly generate a
    /// `u16` value using [`rand::random`].  If the value does not exceed the number of quotes, we
    /// can use it index into vector and pull a quote at random.  If the value exceeds the number
    /// of quotes, we have to roll again.  Lame!
    ///
    /// When we finally have a quote, we send it to the [`ImpKing`] wrapped in a
    /// [`Hijinks::Vandalize`] variant.  This variant includes the quote captured as a string.
    /// We convert the quote to a string using the [`Quote::graffiti`] method.
    /// We include the `name` of the `Imp`, so you can like them on X or something.
    ///
    /// Not being the bravest of species, after sending the [`Hijinks`], the `Imp` goes into
    /// hiding using [`Imp::pause`].
    #[tracing::instrument(skip_all)]
    pub async fn vandalize(&self) -> Arrive<()> {
        let mut idx = 0;
        let mut set = false;
        while !set {
            let trial: u16 = rand::random();
            let trial = trial as usize;
            if trial < self.quotes.len() {
                idx = trial;
                set = true;
            }
        }
        self.tx
            .send(Hijinks::Vandalize(format!(
                "{} says: {}",
                self.name, self.quotes[idx]
            )))
            .await?;
        Self::pause().await;
        Ok(())
    }

    /// The `hijinks` method randomizes `Imp` actions between meddling and vandalization.  The
    /// purpose of this method is to inject some variety into the types of [`Hijinks`] and keep the
    /// user on their toes.
    ///
    /// We use the [`rand::random`] method to do a coin toss, with heads calling the [`Imp::meddle`]
    /// method and tails calling the [`Imp::vandalize`] method.
    #[tracing::instrument(skip_all)]
    pub async fn hijinks(&mut self) -> Arrive<()> {
        loop {
            if rand::random() {
                self.meddle().await?;
            } else {
                self.vandalize().await?;
            }
        }
    }
}

/// The `Hijinks` enum represent the variety of actions that an [`Imp`] can take, and serves as the
/// parent-level message passing mechanism from async -> sync.  In `main.rs`, we create an
/// event loop using [`Hijinks`] as our custom event type:
///
/// ```
/// let event_loop = event_loop::EventLoop::<Hijinks>::with_user_event().build()?;
/// ```
///
/// In turn, we use the [user_event][`winit::application::ApplicationHandler::user_event`] method
/// to alert the application when an imp has sent a new message.
///
/// The purpose of this enum is to let imps perform hijinks.  For the purposes of this library,
/// hijinks refers to:
///
/// * Opening and closing windows.
/// * Logging inspirational quotes at the INFO level.
///
/// I am hoping the distance across the stream from hijinks to useful program operations is but a narrow
/// channel.  Imagine these imps doing things like opening a map in a new window or plotting data onto a chart.
#[derive(Debug)]
pub enum Hijinks {
    /// The `Meddle` variant signals the [`Imp`] wants to take an action.  Currently, this includes
    /// opening or closing a window.  The variant contains an instance of the [`Meddle`] struct,
    /// that contains the action information for interpretation by the application.
    Meddle(Meddle),
    /// The `Vandalize` variant signals that the [`Imp`] wants to log an inspirational quote at the
    /// `INFO` level.  We format the quote as a [`String`] contained in the variant, to streamline
    /// logging the message on the application side.
    Vandalize(String),
    /// The `Filch` variant signals that the [`Imp`] is out of [`Frame`] instances, and is
    /// requesting more.  The [`Filch`] struct contained in the variant holds a transmitter that
    /// the application uses to send back more frames.
    Filch(Filch),
}

/// The `Meddle` struct contains the information necessary for the application to perform the
/// command desired by the imp.  Initially, I just passed around the [`Act`] enum directly.  But
/// then I wanted to include the [`Frame`], but that was only on [`Act::NewWindow`] variants, so it
/// is optional.  Then I wanted to add the name of the imp to the window title, for some flair.
/// That is only needed on `NewWindow` too, but every imp has a name, so I go ahead and pass it
/// in as required.  Not my best work.
#[derive(Debug, Default, Clone, derive_new::new, derive_getters::Getters)]
pub struct Meddle {
    act: Act,
    frame: Option<Frame>,
    title: String,
}

/// The `Filch` struct contains a [`oneshot::Sender`] so the application can send a response to the
/// requestor.  The purpose of this struct is to both signal to the application that the [`Imp`]
/// process is out of [`Frame`] instances, and to provide the application with a means of providing
/// more frames.  This is an example of the SASE pattern described in the docs for [`Imp::filch`].
#[derive(Debug, derive_new::new, derive_getters::Dissolve)]
pub struct Filch {
    tx: oneshot::Sender<Vec<Frame>>,
}

/// The `ImpKing` struct manages async processes in the application.  The purpose of the `ImpKing`
/// is to be the incompetent and unnecessary middle manager of the spiritual realm.  I say
/// unnecessary because everything we are doing here could be done with just threads.  Even within
/// the realm of async, I could spawn [`Imp`] types directly from the application and make his
/// methods redundant.  In the true spirit of pointy-haired bosses, he is staying, and we are just
/// going to let him do his thing.
///
/// As a struct, the `ImpKing` centralizes methods related to creating [`Imp`] types.  To the
/// extent that we have contrived excuses for shuttling data around between the sync and async
/// processes, the `ImpKing` enables us to bridge this gap.
///
/// To call upon the `ImpKing`, use the [`ImpKing::summon`] method.  Once summoned, employ the
/// `ImpKing` using the [`ImpKing::reign`] method.  This method summons a number of [`Imp`] types
/// that run in the background.  The [`Imp`] instances will send [`Hijinks`] messages to the
/// `ImpKing`, where the `ImpKing` passes them along to the application without reading them.
/// He could read them, he just doesn't want to.
///
/// ## Fields ##
///
/// * **frames** - A vector of [`Frame`] instances to pass to [`Imp`] types.
/// * **proxy** - The event loop proxy used to send messages back to the event loop.
/// * **quotes** - Inspirational quotes to pass along to [`Imp`] types.  Imps are not allowed to
///   pass along quotes the `ImpKing` has not already heard.
/// * **rx** - Receiver for [`Hijinks`] from [`Imp`] instances.
/// * **tx** - Transmitter handle passed to an [`Imp`] to perform [`Hijinks`].

#[derive(Debug)]
pub struct ImpKing {
    frames: Vec<Frame>,
    proxy: event_loop::EventLoopProxy<Hijinks>,
    quotes: Quotes,
    rx: mpsc::Receiver<Hijinks>,
    tx: mpsc::Sender<Hijinks>,
}

impl ImpKing {
    /// The `summon` method is the constructor for the `ImpKing`.  The purpose of this method is to
    /// provide an ergonomic way to create an `ImpKing` instance from the parent application.
    /// The caller provides a `proxy` argument of type [`event_loop::EventLoopProxy`] that the
    /// `ImpKing` will use to relay messages back to the application.  The `frames` parameter
    /// provides the reservoir of [`Frame`] instances that the `ImpKing` will give out to the [`Imp`] types.
    /// The `buffer` argument determines the capacity of the [`mpsc::channel`] used to pass
    /// [`Hijinks`] from the [`Imp`] types back to the `ImpKing`.
    ///
    /// First we attempt to read [`Quotes`] from the `data` directory, where there just happens to
    /// be a files called `quotes.csv`.  We deserialize the contents using [`Quotes::from_path`].
    /// We then create the [`mpsc::channel`] passing in `buffer` as the argument, so we can pass
    /// these into the new instance of `ImpKing`.
    #[tracing::instrument(skip_all)]
    pub fn summon(
        proxy: event_loop::EventLoopProxy<Hijinks>,
        buffer: usize,
        frames: Vec<Frame>,
    ) -> Arrive<Self> {
        let path = "/home/erik/code/tardy/data/quotes.csv";
        let quotes = Quotes::from_path(path.into())?;
        let (tx, rx) = mpsc::channel(buffer);
        tracing::info!("Imp King has {} quotes.", quotes.len());
        let imp_king = Self {
            frames,
            proxy,
            quotes,
            rx,
            tx,
        };
        Ok(imp_king)
    }

    /// The `imps` method is the constructor for one or more new [`Imp`] instances.
    /// The purpose of this struct is to enable the `ImpKing` to create minions, so that the
    /// minions can do the hard work of making [`Hijinks`], while he sits back and relaxes.
    /// The method takes a `count` argument specifying the number of [`Imp`] instances to create.
    #[tracing::instrument(skip_all)]
    pub fn imps(&self, count: usize) -> Vec<Imp> {
        let gen = names::Generator::default();
        let names = gen
            .take(count)
            .map(|v| v.to_case(convert_case::Case::Title))
            .collect::<Vec<String>>();
        let mut imps = Vec::new();
        let mut frame_drain = self.frames.clone();
        for name in names.into_iter() {
            let mut frames = Vec::new();
            while frames.len() < FRAMES {
                if let Some(value) = frame_drain.pop() {
                    frames.push(value);
                } else {
                    tracing::warn!("No more frames!");
                    return Vec::new();
                }
            }
            let imp = Imp::new(frames, name, self.quotes.clone(), self.tx.clone());
            imps.push(imp)
        }
        imps
    }

    #[tracing::instrument(skip_all)]
    pub async fn spawn_imps(&self, count: usize) -> Arrive<()> {
        let imps = self.imps(count);
        for mut imp in imps {
            tokio::spawn(async move {
                loop {
                    if imp.hijinks().await.is_err() {
                        break;
                    }
                }
            });
        }
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn listen(&mut self) -> Arrive<()> {
        while let Some(hijinks) = self.rx.recv().await {
            self.proxy.send_event(hijinks)?;
        }
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn reign(&mut self, count: usize) -> Arrive<()> {
        self.spawn_imps(count).await?;
        self.listen().await?;
        Ok(())
    }
}

/// The `Quote` struct contains a single inspirational quote.
/// The purpose of the struct is to embody the relation between the quote and its author.
///
/// * The `author` field identifies the author.  When the attribution is unknown, the value of
///   `author` is `None`.
/// * The `quote` field contains the inspirational saying.
///
/// I struggled to get the [`derive_more::Display`] trait working the way I wanted.  Turns out I
/// can simply pass the custom function back to the fancy macro.  Honestly its not much better than
/// manually writing out the impl for `Display`... but it feels lazier so I am doing it.
#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    derive_more::Display,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "PascalCase")]
#[display("{}", self.graffiti())]
pub struct Quote {
    author: Option<String>,
    quote: String,
}

impl Quote {
    /// The `graffiti` method formats the inspirational quote for display.
    /// The purpose of this method is to combine the quote and author fields in a way that is easy
    /// to read for the user.  If the author attribution is missing, we replace the name with the
    /// value `Unknown`.
    pub fn graffiti(&self) -> String {
        if let Some(author) = &self.author {
            format!("'{}' - {author}", self.quote)
        } else {
            format!("'{}' - Unknown", self.quote)
        }
    }
}

/// The `Quotes` struct is a wrapper around a vector of type [`Quote`].
/// The purpose of this struct is to keep multiple quotes together in a collection.
/// We have implemented the [`derive_more::Deref`], and [`derive_more::DerefMut`] traits to enable
/// easy access to the underlying vector.  This is my favorite way to run the NewType pattern over
/// collections of data, and its use cases seem to crop up a lot in my work.
///
/// The quotes are from some random [gist]("https://gist.github.com/JakubPetriska/") I found when
/// googling for inspirational quotes.
#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    derive_more::Deref,
    derive_more::DerefMut,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Quotes(Vec<Quote>);

impl Quotes {
    /// The `from_path` method attempts to read quotes from a csv file located at `path`.
    /// This is the means by which we bring quotes into the application for use by the [`ImpKing`].
    /// We open the file location using [`fs::File::open`], then attempt to read the csv data using
    /// [`csv::Reader::from_reader`].  For each row of data, we attempt to serialize the contents
    /// into a [`Quote`] struct, and push it into a vector holding the results, which we pass to
    /// user wrapped in a `Quotes` struct.
    ///
    /// Will [`crate::Blame::Io`] if the file path is bad, and will [`crate::Blame::Csv`] if the contents fail to
    /// deserialize into the [`Quote`] type. On second thought, there is no need bubble up an error
    /// if the csv is bad, we will just hand the user a warning.
    pub fn from_path(path: path::PathBuf) -> Arrive<Self> {
        let file = fs::File::open(path)?;
        let mut quotes = Vec::new();
        let mut rdr = csv::Reader::from_reader(file);
        for result in rdr.deserialize() {
            match result {
                Ok(quote) => quotes.push(quote),
                Err(e) => {
                    tracing::warn!("Problem reading quotes: {}", e.to_string());
                }
            }
        }
        Ok(Self(quotes))
    }
}
