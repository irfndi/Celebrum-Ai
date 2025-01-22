import gettext
import os
from functools import lru_cache

SUPPORTED_LANGUAGES = {
    "en": "English",
    "id": "Bahasa Indonesia",
    "es": "Español",
    "ru": "Русский",
    "zh": "中文",
}


class I18n:
    def __init__(self):
        self.translations = {}
        self._load_translations()

    def _load_translations(self):
        """Load all available translations."""
        localedir = os.path.join(os.path.dirname(__file__), "locales")
        for lang in SUPPORTED_LANGUAGES:
            try:
                translation = gettext.translation("base", localedir, languages=[lang])
                self.translations[lang] = translation
            except FileNotFoundError:
                # Fallback to English if translation not found
                self.translations[lang] = gettext.NullTranslations()

    @lru_cache(maxsize=1024)
    def get_text(self, key: str, lang: str = "en") -> str:
        """
        Get translated text for a given key and language.

        Args:
            key (str): The message key to translate
            lang (str): The language code (e.g., 'en', 'es')

        Returns:
            str: The translated text, or the original text if translation not found
        """
        if lang not in self.translations:
            lang = "en"
        return self.translations[lang].gettext(key)

    def get_supported_languages(self) -> dict:
        """Get dictionary of supported languages."""
        return SUPPORTED_LANGUAGES.copy()


# Global instance
i18n = I18n()
