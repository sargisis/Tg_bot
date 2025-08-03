use teloxide::{prelude::*, update_listeners::webhooks};
use teloxide::update_listeners::webhooks::Options;
use teloxide::types::InputFile;
use std::net::SocketAddr;
use std::path::Path;
use teloxide::types::InlineKeyboardButton;
use teloxide::types::InlineKeyboardMarkup;
use std::env;
use dotenv::dotenv;
use std::collections::HashMap;
use teloxide::types::MessageId;
use teloxide::error_handlers::LoggingErrorHandler;
use teloxide::dispatching::UpdateHandler;
use url::Url;

type ChatState = HashMap<ChatId , Vec<MessageId>>;

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    log::info!("Запуск бота с webhook...");

    let bot = Bot::from_env();

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let url = Url::parse(&env::var("WEBHOOK_URL").expect("WEBHOOK_URL must be set")).unwrap();

    let listener = webhooks::axum(bot.clone(), Options::new(addr, url)).await.unwrap();

    let handler: UpdateHandler<teloxide::RequestError> = dptree::entry()
    .branch(Update::filter_message().endpoint(message_handler))
    .branch(Update::filter_callback_query().endpoint(callback_handler));

    Dispatcher::builder(bot, handler)
    .enable_ctrlc_handler()
    .build()
    .dispatch_with_listener(listener, LoggingErrorHandler::with_custom_text("Ошибка в боте"))
    .await;
}

fn main_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("📕 Управляй или подчиняйся", "book1"),
            InlineKeyboardButton::callback("📙 Код Денег", "book2"),
        ],
        vec![
            InlineKeyboardButton::callback("ℹ️ О проекте", "about_project"),
        ],
    ])
}

fn back_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("🔙 Назад", "back")],
    ])
}

async fn message_handler(
    bot: Bot,
    msg: Message,
    state: std::sync::Arc<tokio::sync::Mutex<ChatState>>,
) -> ResponseResult<()> {
    if let Some(text) = msg.text() {
        if text == "/start" {
            let mut state = state.lock().await;

            // Удаляем предыдущие сообщения
            if let Some(ids) = state.get(&msg.chat.id) {
                for &id in ids {
                    let _ = bot.delete_message(msg.chat.id, id).await;
                }
            }
            state.insert(msg.chat.id, Vec::new());

            // Кнопка "Перейти к книгам"
            let keyboard = InlineKeyboardMarkup::new(vec![vec![
                InlineKeyboardButton::callback("📂 Перейти к книгам", "ready"),
            ]]);

            // Текст приветствия
            let welcome_caption = "<b>Привет, друг!</b>\n\
Ты оказался в ReMind-архиве — месте, где мы собираем книги, исчезнувшие с полок, но не из памяти.\n\n\
<b>В этом боте ты сможешь:</b>\n\
1. <b>Приобрести свою книгу.</b> Не по жанру. Не по тренду. А по ощущению.\n\
2. <b>Открой то, что прячут.</b> Мы сохраняем тексты, которые удалили, забыли или пытались спрятать.\n\n\
Нажимай на кнопку ниже и наслаждайся книгами ⬇️";

            // Путь к картинке
            let banner_path = Path::new("assets/welcome.jpg");

            if banner_path.exists() {
                let photo = InputFile::file(banner_path);
                let sent_msg = bot
                    .send_photo(msg.chat.id, photo)
                    .caption(welcome_caption)
                    .parse_mode(teloxide::types::ParseMode::Html)
                    .reply_markup(keyboard)
                    .await?;

                state.get_mut(&msg.chat.id).unwrap().push(sent_msg.id);
            } else {
                let fallback = bot
                    .send_message(msg.chat.id, "Добро пожаловать! (но баннер не найден)")
                    .reply_markup(keyboard)
                    .await?;

                state.get_mut(&msg.chat.id).unwrap().push(fallback.id);
            }
        }
    }
    Ok(())
}

async fn callback_handler(
    bot: Bot,
    q: CallbackQuery,
    state: std::sync::Arc<tokio::sync::Mutex<ChatState>>,
) -> ResponseResult<()> {
    let chat_id = q.message.as_ref().unwrap().chat.id;
    let mut state = state.lock().await;

    // Удаляем все предыдущие сообщения
    if let Some(ids) = state.get(&chat_id) {
        for &id in ids {
            let _ = bot.delete_message(chat_id, id).await;
        }
    }
    state.insert(chat_id, Vec::new());

    if let Some(data) = q.data {
        match data.as_str() {
            "ready" => {
                bot.answer_callback_query(q.id).await?;
                let sent_msg = bot.send_message(chat_id, "📂 Вот то, что мы смогли сохранить:")
                    .reply_markup(main_keyboard())
                    .await?;
                state.get_mut(&chat_id).unwrap().push(sent_msg.id);
            }
            "back" => {
                bot.answer_callback_query(q.id).await?;
                let sent_msg = bot.send_message(chat_id, "📂 Вот то, что мы смогли сохранить:")
                    .reply_markup(main_keyboard())
                    .await?;
                state.get_mut(&chat_id).unwrap().push(sent_msg.id);
            }
            "book1" => {
                bot.answer_callback_query(q.id).await?;
                let sent_msg = bot.send_message(chat_id, 
                    "📕 Управляй или подчиняйся\n\
                    🩸 Книга про влияние, которую не напечатают официально\n\n\
                    📌 Что внутри:\n\
                    – 30 глав о власти, контроле и психологической игре\n\
                    – Техники влияния и манипуляции\n\
                    – Как управлять другими, не поднимая голос\n\
                    – Как не стать жертвой\n\n\
                    📄 Формат: PDF\n\
                    ⏱ Объём: ~150 стр\n\
                    📛 Официально не публиковалась. Распространяется вручную.")
                    .reply_markup(back_keyboard())
                    .await?;
                state.get_mut(&chat_id).unwrap().push(sent_msg.id);

                let file_path = Path::new("books/Управляй или Подчиняйся.pdf");
                if file_path.exists() {
                    let file = InputFile::file(file_path);
                    let sent_file = bot.send_document(chat_id, file)
                        .caption("⚠️ Не распространяйте файл")
                        .await?;
                    state.get_mut(&chat_id).unwrap().push(sent_file.id);
                } else {
                    let sent_err = bot.send_message(chat_id, "❌ Файл временно недоступен").await?;
                    state.get_mut(&chat_id).unwrap().push(sent_err.id);
                }
            }
            "book2" => {
                bot.answer_callback_query(q.id).await?;
                let sent_msg = bot.send_message(chat_id, 
              "📙 Код Денег\n\
                    💰 Эту книгу удалили с форумов. Почему — не говорят.\n\n\
                    📌 Что внутри:\n\
                    – Психология бедности\n\
                    – Финансовое мышление богатых\n\
                    – Примеры от нуля до первого миллиона\n\
                    – Как твои установки управляют твоим счётом\n\n\
                    📄 Формат: PDF\n\
                    ⏱ Объём: ~70 стр\n\
                    📛 Не для публичного доступа. Только здесь."
                   )
                    .reply_markup(back_keyboard())
                    .await?;
                state.get_mut(&chat_id).unwrap().push(sent_msg.id);

                let file_path = Path::new("books/Код Денег.pdf");
                if file_path.exists() {
                    let file = InputFile::file(file_path);
                    let sent_file = bot.send_document(chat_id, file)
                        .caption("⚠️ Только для личного использования")
                        .await?;
                    state.get_mut(&chat_id).unwrap().push(sent_file.id);
                } else {
                    let sent_err = bot.send_message(chat_id, "❌ Файл временно недоступен").await?;
                    state.get_mut(&chat_id).unwrap().push(sent_err.id);
                }
            }
            "about_project" => {
                bot.answer_callback_query(q.id).await?;
                let sent_msg = bot.send_message(chat_id, 
                    "Есть книги, которые не найти в магазинах.
                    Их нет на полках, нет в рекламе, нет в поиске.
                    Они исчезают. Их удаляют.
                    Иногда — потому что они опасны. Иногда — потому что слишком честны.
                    А иногда — потому что кто-то решил, что ты не должен их читать.

                    Этот проект — не просто библиотека.
                    Это архив из теней.
                    Мы собираем, восстанавливаем и сохраняем книги, которые вычищают из интернета.
                    Некоторые были удалены. Некоторые — запрещены. Некоторые — никогда и не должны были появиться в открытом доступе.

                    Каждая книга здесь — это не просто PDF.
                    Это знание, которое выжило.
                    Знание, которое может изменить мышление, сломать старые рамки и показать то, что обычно скрывают.

                    📅 Мы добавляем одну новую книгу каждую неделю.
                    Она появляется здесь — тихо, без рекламы, без шума.
                    И если ты читаешь это — ты успел. Пока не стало поздно.

                    Подписывайся на наш открытый канал, чтобы не пропустить следующие книги.
                    Мы не обещаем, что они будут всегда.
                    Но пока мы здесь — они будут появляться.
                    🔗 @Remind_ofc")
                    .reply_markup(back_keyboard())
                    .await?;
                state.get_mut(&chat_id).unwrap().push(sent_msg.id);
            }
            _ => {}
        }
    }
    Ok(())
}