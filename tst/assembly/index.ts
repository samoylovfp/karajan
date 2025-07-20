// The entry file of your WebAssembly module.


@external("host", "sendMessage")
declare function sendMessage(chat_id: i64, message: string): void

class Update {
  message: Message | null = null;
}

class Chat {
  id: i64 = 0;
}

class User {
  id: i64 = 0;
  first_name: string = "";
  last_name: string = "";
}

class Message {
  id: i64 = 0;
  chat: Chat = { id: 0 };
  from: User | null = null;
  text: string | null = null;
}

export function processUpdate(update: Update): void {
  if (update.message) {
    let message = (update.message as Message)
    sendMessage(message.chat.id, buildReply(message))
  }
}

function buildReply(message: Message): string {
  let from = message.from != null ? (message.from as User).first_name : "Unknown"
  return `Hello, ${from}, your id is ${message.chat.id}, you said ${message.text}`
}
