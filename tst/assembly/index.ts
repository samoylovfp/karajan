import { JSON } from "json-as";

@json
class Update {
  id: i64 = -1;
  message: Message | null = null;
}

@json
class Message {
  text: String | null = null;
  chat: Chat = { id: -1 };
  from: User | null = null;
}

@json
class User {
  id: i64 = -1;
  first_name: string = "";
  last_name: string | null = null;
}

@json
class Chat {
  id: i64 = -1;
}

@external("host", "sendMessage")
declare function sendMessage(chat_id: i64, message: string): void

export function processUpdate(update: string): void {
  let update_json: Update = JSON.parse<Update>(update);
  if (update_json.message) {
    let message = (update_json.message as Message)
    sendMessage(message.chat.id, buildReply(message))
  }
}

function buildReply(message: Message): string {
  let from = message.from != null ? (message.from as User).first_name : "Unknown"
  return `Hello, ${from}, your id is ${message.chat.id}, you said ${message.text}`
}
