import unittest
from meetsmatch.i18n import I18n, SUPPORTED_LANGUAGES


class TestI18n(unittest.TestCase):
    def setUp(self):
        self.i18n = I18n()

    def test_supported_languages(self):
        """Test that supported languages are correctly defined."""
        languages = self.i18n.get_supported_languages()
        self.assertEqual(languages, SUPPORTED_LANGUAGES)
        self.assertIn("en", languages)
        self.assertIn("id", languages)

    def test_get_text_default_language(self):
        """Test getting text in default language (English)."""
        text = self.i18n.get_text("Welcome to MeetsMatch! Let's set up your profile.")
        self.assertEqual(text, "Welcome to MeetsMatch! Let's set up your profile.")

    def test_get_text_fallback(self):
        """Test fallback to English for unsupported language."""
        text = self.i18n.get_text(
            "Welcome to MeetsMatch! Let's set up your profile.", "xx"
        )
        self.assertEqual(text, "Welcome to MeetsMatch! Let's set up your profile.")

    def test_cache_behavior(self):
        """Test that caching works correctly."""
        key = "Welcome to MeetsMatch! Let's set up your profile."
        # First call
        text1 = self.i18n.get_text(key)
        # Second call (should hit cache)
        text2 = self.i18n.get_text(key)
        self.assertEqual(text1, text2)
