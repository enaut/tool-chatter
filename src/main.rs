use std::{
    collections::HashMap,
    fmt::Display,
    hash::Hash,
    io::{self, BufRead},
};

use chrono::{Duration, NaiveDateTime};

/// Representing a BigBlueButton meeting most of the things are omitted and just meeting_id the approximate time and the (private and public)chats.
struct Meeting {
    meeting_id: String,
    time: NaiveDateTime,
    chats: HashMap<String, Chat>,
}

impl Hash for Meeting {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.meeting_id.hash(state);
    }
}

impl PartialEq for Meeting {
    fn eq(&self, other: &Self) -> bool {
        self.meeting_id.eq(&other.meeting_id)
    }
}

impl Display for Meeting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\n{}\n\n{} - {}\n",
            "#".repeat(80),
            self.time.format("%d.%m.%Y %H:%M"),
            self.meeting_id
        )?;
        for (_, chat) in &self.chats {
            write!(f, "{}", chat)?;
        }
        Ok(())
    }
}

/// A BigBlueButton chat - either private or public. Most of the parameters are omitted so it is mostly a collection of the contained messages.
#[derive(Eq, Hash, PartialEq, Clone)]
struct Chat {
    chat_id: String,
    messages: Vec<Message>,
}
impl Display for Chat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n{}\n{}\n", "_".repeat(80), self.chat_id)?;
        for msg in &self.messages {
            write!(f, "  {}\n", msg)?;
        }
        Ok(())
    }
}
/// Representing one chatmessage in BigBlueButton
#[derive(Eq, Hash, PartialEq, Clone)]
struct Message {
    author: String,
    message: String,
    time: NaiveDateTime,
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "    {}:{:.>15}: {}",
            self.time.format("%H:%M"),
            self.author,
            self.message
        )
    }
}

/// Parse the stdin (incomming pipe) and print nicely formatted chatmessages ordered by their meetings and their chatroom.
///
/// Note: if the order of the messages isnt consequential the meeting times will be slightly wrong
fn main() {
    // get the pipe
    let stdin = io::stdin();
    // the collection of meetings.
    let mut meetings: HashMap<String, Meeting> = HashMap::new();

    // iterate over all the loglines found in the input
    for line in stdin.lock().lines() {
        let line = line.expect("Could not read line from standard in");
        // skip the first part (date and processinformation and go to the first brace which is the starting of the json log)
        if let Some(start_pos) = line.find("{") {
            let data = json::parse(&line[start_pos..]);
            match data {
                Ok(data) => {
                    // First get the time of the message. The timestamp is in milliseconds since epoch.
                    let secs = data["envelope"]["timestamp"].as_i64().expect(&format!(
                        "{} is not a number",
                        &data["envelope"]["timestamp"]
                    ));
                    let time = NaiveDateTime::from_timestamp(0, 0) + Duration::milliseconds(secs);

                    // get the meeting_id and create the meeting if it does not exist yet.
                    let meeting_id = data["envelope"]["routing"]["meetingId"].to_string();
                    if !meetings.contains_key(&meeting_id) {
                        let meeting = Meeting {
                            meeting_id: meeting_id.clone(),
                            time,
                            chats: HashMap::new(),
                        };
                        println!("inserting: {}", &meeting_id);
                        meetings.insert(meeting_id.to_string(), meeting);
                    }

                    // either get the newly created or the already existing meeting
                    let meeting = meetings
                        .get_mut(&meeting_id)
                        .expect("there should be a meeting");

                    // create the chat
                    let body = &data["core"]["body"];
                    let chat_id = body["chatId"].to_string();

                    // check if the chat already exists if not create it
                    if !meeting.chats.contains_key(&chat_id) {
                        let chat = Chat {
                            chat_id: chat_id.clone(),
                            messages: Vec::new(),
                        };
                        meeting.chats.insert(chat_id.clone(), chat);
                    }

                    // get the information on the message
                    let msg = &body["msg"];
                    let sender = msg["sender"]["name"].to_string();
                    let message = msg["message"].to_string();
                    // add the message to the list
                    let chat = meeting.chats.get_mut(&chat_id).unwrap();
                    chat.messages.push(Message {
                        author: sender.clone(),
                        message,
                        time,
                    });
                    // every message is twice in the logs which is why the messages are deduped. This could be done more performant at a different place but it was not an issue with my problem.
                    chat.messages.dedup_by(|s, o| s.message == o.message);
                }
                Err(e) => {
                    // If for some reason the message could not be parsed print the message at the very beginning.
                    println!("{}\n{}", e, &line[start_pos..])
                }
            }
        } else {
            println!("{}", line);
        }
    }

    // print everything to stdout
    println!("{}", meetings.len());
    for (_, meeting) in meetings {
        println!("\n\n{}", meeting);
    }
}
