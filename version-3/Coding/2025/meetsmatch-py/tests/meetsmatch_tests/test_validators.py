import pytest
from unittest.mock import patch
from meetsmatch.validators import MediaValidator, RateLimiter, Cache
from PIL import Image
import io
import asyncio


@pytest.mark.asyncio
class TestMediaValidator:
    def setup_method(self):
        self.validator = MediaValidator()

    async def test_get_mime_type(self):
        """Test MIME type detection from file names."""
        assert self.validator.get_mime_type("test.jpg") == "image/jpeg"
        assert self.validator.get_mime_type("test.png") == "image/png"
        assert self.validator.get_mime_type("test.mp4") == "video/mp4"

    @patch("magic.from_buffer")
    async def test_validate_file_type(self, mock_magic):
        """Test file type validation."""
        # Test valid image
        mock_magic.return_value = "image/jpeg"
        result, file_type = await self.validator.validate_file_type(
            b"fake_data", "test.jpg"
        )
        assert result
        assert file_type == "image"

        # Test mismatched type
        mock_magic.return_value = "image/png"
        result, message = await self.validator.validate_file_type(
            b"fake_data", "test.jpg"
        )
        assert not result
        assert "does not match" in message

    async def test_validate_image_dimensions(self):
        """Test image dimension validation."""
        # Create test image
        img = Image.new("RGB", (300, 300))
        img_bytes = io.BytesIO()
        img.save(img_bytes, format="JPEG")

        result, message = await self.validator.validate_image(img_bytes.getvalue())
        assert result

        # Test small image
        img = Image.new("RGB", (100, 100))
        img_bytes = io.BytesIO()
        img.save(img_bytes, format="JPEG")

        result, message = await self.validator.validate_image(img_bytes.getvalue())
        assert not result
        assert "too small" in message.lower()

    async def test_validate_file_size(self):
        """Test file size validation."""
        # Test valid image size
        result, message = await self.validator.validate_file_size(1024 * 1024, "image")
        assert result

        # Test oversized image
        result, message = await self.validator.validate_file_size(
            10 * 1024 * 1024, "image"
        )
        assert not result
        assert "too large" in message.lower()


@pytest.mark.asyncio
class TestRateLimiter:
    def setup_method(self):
        self.limiter = RateLimiter()

    async def test_message_rate_limit(self):
        """Test message rate limiting."""
        user_id = 1

        # Should allow initial messages
        for _ in range(30):
            allowed, _ = await self.limiter.check_rate_limit(user_id, "message")
            assert allowed

        # Should block after limit
        allowed, wait_time = await self.limiter.check_rate_limit(user_id, "message")
        assert not allowed
        assert wait_time > 0

    async def test_different_actions(self):
        """Test rate limiting for different action types."""
        user_id = 1

        # Message should not affect media limit
        for _ in range(30):
            await self.limiter.check_rate_limit(user_id, "message")

        allowed, _ = await self.limiter.check_rate_limit(user_id, "media")
        assert allowed


@pytest.mark.asyncio
class TestCache:
    def setup_method(self):
        self.cache = Cache(max_size=2, ttl=1)

    async def test_cache_set_get(self):
        """Test basic cache operations."""
        await self.cache.set("key1", "value1")
        value = await self.cache.get("key1")
        assert value == "value1"

    async def test_cache_max_size(self):
        """Test cache size limit."""
        await self.cache.set("key1", "value1")
        await self.cache.set("key2", "value2")
        await self.cache.set("key3", "value3")  # Should evict key1

        assert await self.cache.get("key1") is None
        assert await self.cache.get("key2") == "value2"
        assert await self.cache.get("key3") == "value3"

    async def test_cache_expiry(self):
        """Test cache entry expiration."""
        await self.cache.set("key1", "value1")
        await asyncio.sleep(1.1)  # Wait for TTL to expire
        assert await self.cache.get("key1") is None


if __name__ == "__main__":
    pytest.main()
