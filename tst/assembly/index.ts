// The entry file of your WebAssembly module.


@external("host", "sendMessage")
declare function sendMessage(chat_id: i64, message: string): void

import { Update } from "../../gen/tg.ts";

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
