import logging
import json
from .matching_enhanced import enhanced_matcher
from .models import User, Session, Interaction
from .config import Config
from .validators import media_validator, rate_limiter, cache
from .reports import report_manager
from .account import account_manager
from .media import media_handler

from telegram import (
    Update,
    ReplyKeyboardMarkup,
    InlineKeyboardButton,
    InlineKeyboardMarkup,
)
from telegram.ext import (
    Application,
    CommandHandler,
    MessageHandler,
    filters,
    ConversationHandler,
    ContextTypes,
    CallbackQueryHandler,
)

# Define conversation states
(
    REGISTER_NAME,
    REGISTER_GENDER,
    REGISTER_AGE,
    REGISTER_BIO,
    REGISTER_LOCATION,
    REGISTER_INTERESTS,
) = range(6)

logging.basicConfig(
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s", level=logging.INFO
)
logger = logging.getLogger(__name__)


class Bot:
    def __init__(self):
        self.i18n = None  # Placeholder for i18n

    async def get_user(
        self, update: Update, context: ContextTypes.DEFAULT_TYPE
    ) -> User | None:
        session = Session()
        try:
            user = (
                session.query(User)
                .filter_by(telegram_id=update.effective_user.id)
                .first()
            )
            return user
        finally:
            session.close()

    async def start(self, update: Update, context: ContextTypes.DEFAULT_TYPE) -> int:
        """Start the conversation and check user's profile status."""
        if not update.message:
            return

        user_id = update.message.from_user.id
        allowed, wait_time = await rate_limiter.check_rate_limit(user_id, "message")
        if not allowed:
            await update.message.reply_text(
                f"Please wait {wait_time} seconds before sending more messages."
            )
            return

        session = Session()
        try:
            user = session.query(User).filter_by(telegram_id=user_id).first()

            if not user:
                # New user - start registration
                user = User(
                    telegram_id=user_id, username=update.message.from_user.username
                )
                session.add(user)
                session.commit()
                await self.start_registration(update, context)
                return

            if not user.is_profile_complete():
                await update.message.reply_text("Please complete your registration.")
                await self.continue_registration(update, context, user)
                return

            # Check for likes
            likes = (
                session.query(Interaction)
                .filter(
                    Interaction.target_user_id == user.id,
                    Interaction.interaction_type == "like",
                )
                .all()
            )

            if likes:
                await update.message.reply_text(
                    "Welcome back! Your profile is complete."
                )
                await self.show_likes(update, context, likes)
            else:
                await update.message.reply_text(
                    "Welcome back! Your profile is complete."
                )
                await update.message.reply_text("No likes yet! Here's the main menu.")
                keyboard = [["/match", "/profile"], ["/config"]]
                reply_markup = ReplyKeyboardMarkup(keyboard, one_time_keyboard=True)
                await update.message.reply_text("Main Menu:", reply_markup=reply_markup)

        finally:
            session.close()

    async def handle_media(self, update: Update, context: ContextTypes.DEFAULT_TYPE):
        """Handle media file uploads."""
        if not update.message:
            return

        user_id = update.message.from_user.id

        # Check rate limit
        allowed, wait_time = await rate_limiter.check_rate_limit(user_id, "media")
        if not allowed:
            await update.message.reply_text(
                f"Please wait {wait_time} seconds before uploading more media."
            )
            return

        # Get file
        file = (
            update.message.photo[-1]
            if update.message.photo
            else update.message.video
            if update.message.video
            else None
        )

        if not file:
            await update.message.reply_text("Please send a photo or video.")
            return

        try:
            # Download file
            file_obj = await context.bot.get_file(file.file_id)
            file_data = await file_obj.download_as_bytearray()

            # Validate file
            is_valid, message = await media_validator.validate_file_type(
                file_data, file_obj.file_path
            )
            if not is_valid:
                await update.message.reply_text(message)
                return

            file_type = message  # 'image' or 'video'

            # Validate size
            is_valid, message = await media_validator.validate_file_size(
                len(file_data), file_type
            )
            if not is_valid:
                await update.message.reply_text(message)
                return

            # If it's an image, validate dimensions
            if file_type == "image":
                is_valid, message = await media_validator.validate_image(file_data)
                if not is_valid:
                    await update.message.reply_text(message)
                    return

            # Save media
            success, message = await media_handler.save_media(
                user_id, file.file_id, file_type, len(file_data)
            )

            await update.message.reply_text(
                "Media uploaded successfully!"
                if success
                else f"Error uploading media: {message}"
            )

        except Exception as e:
            await update.message.reply_text(f"Error processing media: {str(e)}")

    async def handle_match(self, update: Update, context: ContextTypes.DEFAULT_TYPE):
        """Handle match command."""
        if not update.message:
            return

        user_id = update.message.from_user.id

        # Check rate limit
        allowed, wait_time = await rate_limiter.check_rate_limit(user_id, "match")
        if not allowed:
            await update.message.reply_text(
                f"Please wait {wait_time} seconds before requesting more matches."
            )
            return

        session = Session()
        try:
            user = session.query(User).filter_by(telegram_id=user_id).first()
            if not user or not user.is_profile_complete():
                await update.message.reply_text("Please complete your profile first.")
                return

            # Try to get matches from cache
            cache_key = f"matches:{user_id}"
            matches = await cache.get(cache_key)

            if not matches:
                # Get fresh matches
                matches = await enhanced_matcher.find_matches(user)
                await cache.set(cache_key, matches)

            if not matches:
                await update.message.reply_text(
                    "No matches found at this time. Try again later!"
                )
                return

            # Show first match
            await self.show_profile(update, context, matches[0][0])
            context.user_data["current_matches"] = matches
            context.user_data["current_match_index"] = 0

        finally:
            session.close()

    async def handle_report(self, update: Update, context: ContextTypes.DEFAULT_TYPE):
        """Handle report command."""
        if not update.message:
            return

        user_id = update.message.from_user.id

        # Check rate limit
        allowed, wait_time = await rate_limiter.check_rate_limit(user_id, "report")
        if not allowed:
            await update.message.reply_text(
                f"Please wait {wait_time} seconds before submitting more reports."
            )
            return

        if "current_matches" not in context.user_data:
            await update.message.reply_text("No profile to report.")
            return

        matches = context.user_data["current_matches"]
        index = context.user_data.get("current_match_index", 0)

        if 0 <= index < len(matches):
            reported_user = matches[index][0]
            success, message = await report_manager.report_user(
                user_id, reported_user.id, "inappropriate_content"
            )

            await update.message.reply_text(message)

            # Move to next match if available
            if index + 1 < len(matches):
                context.user_data["current_match_index"] = index + 1
                await self.show_profile(update, context, matches[index + 1][0])
        else:
            await update.message.reply_text("No profile to report.")

    async def handle_delete_account(
        self, update: Update, context: ContextTypes.DEFAULT_TYPE
    ):
        """Handle account deletion request."""
        if not update.message:
            return

        user_id = update.message.from_user.id
        success, message = await account_manager.request_deletion(user_id)

        await update.message.reply_text(message)

        if success:
            # Clear user data
            context.user_data.clear()
            await update.message.reply_text(
                "Your account has been scheduled for deletion. "
                "You can cancel this within 30 days by using /cancel_deletion"
            )

    async def start_registration(
        self, update: Update, context: ContextTypes.DEFAULT_TYPE
    ):
        await update.message.reply_text(
            "Welcome to MeetsMatch! Let's create your profile."
        )
        await update.message.reply_text("What's your name?")
        return REGISTER_NAME

    async def continue_registration(
        self, update: Update, context: ContextTypes.DEFAULT_TYPE, user: User
    ):
        print(
            "Debugging continue_registration: "
            f"name={user.name}, "
            f"gender={user.gender}, "
            f"age={user.age}, "
            f"bio={user.bio}, "
            f"location={user.location}, "
            f"interests={user.interests}"
        )
        if not user.name:
            print("User's name is missing")
            await update.message.reply_text("What's your name?")
            return REGISTER_NAME
        elif not user.gender:
            print("User's gender is missing")
            await update.message.reply_text("What's your gender? (Male/Female)")
            return REGISTER_GENDER
        elif not user.age:
            await update.message.reply_text("How old are you? (12-100)")
            return REGISTER_AGE
        elif not user.bio:
            await update.message.reply_text(
                "Tell us about yourself (max 120 characters):"
            )
            return REGISTER_BIO
        elif not user.location:
            await update.message.reply_text("Where are you located? (City, Country)")
            return REGISTER_LOCATION
        elif not user.interests:
            await update.message.reply_text(
                "What are your interests? (comma separated)"
            )
            return REGISTER_INTERESTS

    async def show_main_menu(self, update: Update, context: ContextTypes.DEFAULT_TYPE):
        user = await self.get_user(update, context)
        if not user:
            return

        keyboard = [
            [
                InlineKeyboardButton(
                    self.i18n.t("match", user.language), callback_data="match"
                )
            ],
            [
                InlineKeyboardButton(
                    self.i18n.t("profile", user.language), callback_data="profile"
                )
            ],
            [
                InlineKeyboardButton(
                    self.i18n.t("configuration", user.language),
                    callback_data="configuration",
                )
            ],
        ]
        reply_markup = InlineKeyboardMarkup(keyboard)
        await context.bot.send_message(
            chat_id=update.effective_chat.id,
            text=self.i18n.t("main_menu", user.language),
            reply_markup=reply_markup,
        )

    async def show_likes(
        self, update: Update, context: ContextTypes.DEFAULT_TYPE, likes: list
    ):
        for like in likes:
            await update.message.reply_text(
                f"You have been liked by {like.user.username}!"
            )

    async def show_profile(
        self, update: Update, context: ContextTypes.DEFAULT_TYPE, user: User
    ):
        profile_text = f"""
    Name: {user.name}
    Age: {user.age}
    Gender: {user.gender}
    Location: {user.location}
    Bio: {user.bio}
    Interests: {', '.join(json.loads(user.interests))}
    """
        await update.message.reply_text(profile_text)

    async def register_name(
        self, update: Update, context: ContextTypes.DEFAULT_TYPE
    ) -> int:
        name = update.message.text.strip()
        if not name:
            await update.message.reply_text("Please enter a valid name.")
            return REGISTER_NAME

        session = Session()
        user = (
            session.query(User).filter_by(telegram_id=update.effective_user.id).first()
        )
        user.name = name
        session.commit()

        await update.message.reply_text("What's your gender? (Male/Female/Other)")
        return REGISTER_GENDER

    async def register_gender(
        self, update: Update, context: ContextTypes.DEFAULT_TYPE
    ) -> int:
        gender = update.message.text.strip()
        if gender not in ["Male", "Female", "Other"]:
            await update.message.reply_text("Please enter Male, Female, or Other.")
            return REGISTER_GENDER

        session = Session()
        user = (
            session.query(User).filter_by(telegram_id=update.effective_user.id).first()
        )
        user.gender = gender
        session.commit()

        await update.message.reply_text("What's your age? (Please enter a number)")
        return REGISTER_AGE

    async def register_age(
        self, update: Update, context: ContextTypes.DEFAULT_TYPE
    ) -> int:
        try:
            age = int(update.message.text.strip())
            if age < 18 or age > 100:
                raise ValueError()
        except ValueError:
            await update.message.reply_text(
                "Please enter a valid age between 18 and 100."
            )
            return REGISTER_AGE

        session = Session()
        user = (
            session.query(User).filter_by(telegram_id=update.effective_user.id).first()
        )
        user.age = age
        session.commit()

        await update.message.reply_text("Tell us a bit about yourself:")
        return REGISTER_BIO

    async def register_bio(
        self, update: Update, context: ContextTypes.DEFAULT_TYPE
    ) -> int:
        bio = update.message.text.strip()
        session = Session()
        user = (
            session.query(User).filter_by(telegram_id=update.effective_user.id).first()
        )
        user.bio = bio
        session.commit()

        await update.message.reply_text("What city are you in?")
        return REGISTER_LOCATION

    async def register_location(
        self, update: Update, context: ContextTypes.DEFAULT_TYPE
    ) -> int:
        location = update.message.text.strip()
        session = Session()
        user = (
            session.query(User).filter_by(telegram_id=update.effective_user.id).first()
        )
        user.location = location
        session.commit()

        await update.message.reply_text("What are your interests? (Comma separated)")
        return REGISTER_INTERESTS

    async def register_interests(
        self, update: Update, context: ContextTypes.DEFAULT_TYPE
    ) -> int:
        interests = [i.strip() for i in update.message.text.split(",")]
        session = Session()
        user = (
            session.query(User).filter_by(telegram_id=update.effective_user.id).first()
        )
        user.interests = json.dumps(interests)
        session.commit()

        await update.message.reply_text(
            "Great! Your profile is complete. Let's start matching!"
        )
        return ConversationHandler.END

    async def cancel(self, update: Update, context: ContextTypes.DEFAULT_TYPE):
        await update.message.reply_text("Registration cancelled.")
        return ConversationHandler.END


if __name__ == "__main__":
    try:
        bot = Bot()
        app = Application.builder().token(Config.BOT_TOKEN).build()
    except Exception as e:
        logger.error(f"Failed to initialize bot: {e}")
        exit(1)

    # Setup conversation handler for registration
    conv_handler = ConversationHandler(
        entry_points=[CommandHandler("start", bot.start)],
        states={
            REGISTER_NAME: [
                MessageHandler(filters.TEXT & ~filters.COMMAND, bot.register_name)
            ],
            REGISTER_GENDER: [
                MessageHandler(filters.TEXT & ~filters.COMMAND, bot.register_gender)
            ],
            REGISTER_AGE: [
                MessageHandler(filters.TEXT & ~filters.COMMAND, bot.register_age)
            ],
            REGISTER_BIO: [
                MessageHandler(filters.TEXT & ~filters.COMMAND, bot.register_bio)
            ],
            REGISTER_LOCATION: [
                MessageHandler(filters.TEXT & ~filters.COMMAND, bot.register_location)
            ],
            REGISTER_INTERESTS: [
                MessageHandler(filters.TEXT & ~filters.COMMAND, bot.register_interests)
            ],
        },
        fallbacks=[CommandHandler("cancel", bot.cancel)],
    )

    app.add_handler(conv_handler)
    app.add_handler(CommandHandler("match", bot.handle_match))
    app.add_handler(CommandHandler("report", bot.handle_report))
    app.add_handler(CommandHandler("delete_account", bot.handle_delete_account))
    app.add_handler(MessageHandler(filters.MEDIA & ~filters.COMMAND, bot.handle_media))
    app.add_handler(CallbackQueryHandler(bot.show_main_menu, pattern="main_menu"))

    app.run_polling()
