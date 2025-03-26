"use strict";

var chat_view;

async function submit_form(event) {
    event.preventDefault();
    const fields = event.target.elements;
    const sender = fields['sender'].value;
    const body = fields['body'].value;
    let new_message = {
        sender, body,
    };
    fields['body'].value = '';
    await fetch("/api/chat/post/json", {
        method: "POST",
        body: JSON.stringify(new_message),
        headers: {
            "Content-Type": "application/json",
        },
    });
    wipe_and_refresh_chat();
}

function ready_posting_form() {
    const form = document.getElementById("PostingForm");
    form.addEventListener("submit", submit_form);
}

function show_message(message, view) {
    const article = document.createElement('article');
    const metadata = document.createElement('p');
    const by_line = document.createElement('span');
    by_line.classList.add(['byLine'])
    by_line.innerText = message.sender;
    const time_line = document.createElement('span');
    const time = document.createElement('time');
    time.datetime = message.date.toISOString();
    const time_str = message.date.toLocaleTimeString(navigator.language);
    time.appendChild(new Text(time_str));
    time_line.classList.add(['timeLine'])
    time_line.appendChild(time);
    metadata.classList.add(['metadata'])
    metadata.appendChild(by_line);
    metadata.appendChild(new Text(' '));
    metadata.appendChild(time_line);
    const body = document.createElement('p');
    // body.classList.add(['body']);
    body.appendChild(new Text(message.body));
    article.classList.add(["message"]);
    article.appendChild(metadata);
    article.appendChild(body);
    if (view) {
        view.appendChild(article);
    }
    return article;
}

async function load_chat() {
    const api_messages = await fetch("/api/chat/get/all");
    if (api_messages.ok) {
        let all_messages = await api_messages.json();
        let processed_messages = new Array();
        for (const message of all_messages) {
            processed_messages.push({ ...message, date: new Date(Date.parse(message.time)) });
        }
        return processed_messages;
    } else {
        return null;
    }
}

function render_chat(messages) {
    for (const message of messages) {
        show_message(message, chat_view);
    }
}

var refresh_timeout = null;
async function wipe_and_refresh_chat() {
    if (refresh_timeout) {
        clearTimeout(refresh_timeout);
        refresh_timeout = null;
    }
    let chats = await load_chat();
    if (chats) {
        chat_view.textContent = ''; // Remove all the children in chat_view.
        render_chat(chats);
    }
    refresh_timeout = setTimeout(wipe_and_refresh_chat, 5000);
}

function main() {
    chat_view = document.getElementById("ChatView");
    ready_posting_form();
    wipe_and_refresh_chat();
}

main()
