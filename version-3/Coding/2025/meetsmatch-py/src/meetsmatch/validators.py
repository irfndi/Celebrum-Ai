from typing import Tuple, Optional
import mimetypes
import magic
from PIL import Image
import io
from functools import lru_cache
from datetime import datetime, timedelta


class MediaValidator:
    def __init__(self):
        self.max_image_size_bytes = 5 * 1024 * 1024  # 5MB
        self.max_video_size_bytes = 20 * 1024 * 1024  # 20MB
        self.max_files_per_user = 5
        self.allowed_image_types = {"image/jpeg", "image/png", "image/webp"}
        self.allowed_video_types = {"video/mp4", "video/quicktime"}
        self.min_image_dimensions = (200, 200)
        self.max_image_dimensions = (4096, 4096)

    @lru_cache(maxsize=1000)
    def get_mime_type(self, file_name: str) -> str:
        """Get MIME type from file name."""
        return mimetypes.guess_type(file_name)[0]

    async def validate_file_type(
        self, file_data: bytes, file_name: str
    ) -> Tuple[bool, str]:
        """
        Validate file type using both extension and content analysis.

        Args:
            file_data: Raw file data
            file_name: Original file name

        Returns:
            Tuple[bool, str]: (is_valid, error_message)
        """
        # Check by extension
        mime_type = self.get_mime_type(file_name)
        if not mime_type:
            return False, "Unsupported file type"

        # Check by content
        try:
            actual_type = magic.from_buffer(file_data, mime=True)
            if actual_type != mime_type:
                return False, "File type does not match extension"
        except Exception as e:
            return False, f"Error validating file type: {str(e)}"

        # Validate against allowed types
        if mime_type in self.allowed_image_types:
            return True, "image"
        elif mime_type in self.allowed_video_types:
            return True, "video"
        else:
            return False, "Unsupported file type"

    async def validate_image(self, image_data: bytes) -> Tuple[bool, str]:
        """
        Validate image dimensions and format.

        Args:
            image_data: Raw image data

        Returns:
            Tuple[bool, str]: (is_valid, error_message)
        """
        try:
            img = Image.open(io.BytesIO(image_data))
            width, height = img.size

            # Check dimensions
            if (
                width < self.min_image_dimensions[0]
                or height < self.min_image_dimensions[1]
            ):
                return (
                    False,
                    f"Image too small. Minimum dimensions: {self.min_image_dimensions}",
                )

            if (
                width > self.max_image_dimensions[0]
                or height > self.max_image_dimensions[1]
            ):
                return (
                    False,
                    f"Image too large. Maximum dimensions: {self.max_image_dimensions}",
                )

            return True, "Valid image"

        except Exception as e:
            return False, f"Error validating image: {str(e)}"

    async def validate_file_size(
        self, file_size: int, file_type: str
    ) -> Tuple[bool, str]:
        """
        Validate file size based on type.

        Args:
            file_size: Size in bytes
            file_type: 'image' or 'video'

        Returns:
            Tuple[bool, str]: (is_valid, error_message)
        """
        if file_type == "image" and file_size > self.max_image_size_bytes:
            return False, (
                f"Image too large. Maximum size: "
                f"{self.max_image_size_bytes // 1024 // 1024}MB"
            )

        if file_type == "video" and file_size > self.max_video_size_bytes:
            return False, (
                f"Video too large. Maximum size: "
                f"{self.max_video_size_bytes // 1024 // 1024}MB"
            )

        return True, "Valid size"


class RateLimiter:
    def __init__(self):
        self.limits = {
            "message": {"count": 30, "window": 60},  # 30 messages per minute
            "media": {"count": 5, "window": 60},  # 5 media uploads per minute
            "match": {"count": 20, "window": 60},  # 20 match requests per minute
            "report": {"count": 5, "window": 3600},  # 5 reports per hour
        }
        self.user_actions = {}

    async def check_rate_limit(
        self, user_id: int, action_type: str
    ) -> Tuple[bool, Optional[int]]:
        """
        Check if action is within rate limits.

        Args:
            user_id: User ID
            action_type: Type of action ('message', 'media', 'match', 'report')

        Returns:
            Tuple[bool, Optional[int]]: (is_allowed, seconds_until_reset)
        """
        now = datetime.utcnow()
        key = f"{user_id}:{action_type}"

        if key not in self.user_actions:
            self.user_actions[key] = []

        # Clean old actions
        window = timedelta(seconds=self.limits[action_type]["window"])
        self.user_actions[key] = [
            ts for ts in self.user_actions[key] if now - ts < window
        ]

        # Check limit
        if len(self.user_actions[key]) >= self.limits[action_type]["count"]:
            oldest = min(self.user_actions[key])
            reset_time = oldest + window
            seconds_left = int((reset_time - now).total_seconds())
            return False, seconds_left

        # Add new action
        self.user_actions[key].append(now)
        return True, None


class Cache:
    def __init__(self, max_size: int = 1000, ttl: int = 3600):
        self.max_size = max_size
        self.ttl = ttl
        self.cache = {}
        self.access_times = {}

    async def get(self, key: str) -> Optional[any]:
        """Get value from cache if not expired."""
        if key in self.cache:
            now = datetime.utcnow()
            if now - self.access_times[key] < timedelta(seconds=self.ttl):
                self.access_times[key] = now
                return self.cache[key]
            else:
                del self.cache[key]
                del self.access_times[key]
        return None

    async def set(self, key: str, value: any) -> None:
        """Set value in cache with cleanup if needed."""
        now = datetime.utcnow()

        # Clean expired entries
        expired = [
            k
            for k, t in self.access_times.items()
            if now - t > timedelta(seconds=self.ttl)
        ]
        for k in expired:
            del self.cache[k]
            del self.access_times[k]

        # Clean oldest if at max size
        if len(self.cache) >= self.max_size:
            oldest = min(self.access_times.items(), key=lambda x: x[1])[0]
            del self.cache[oldest]
            del self.access_times[oldest]

        # Set new value
        self.cache[key] = value
        self.access_times[key] = now


# Global instances
media_validator = MediaValidator()
rate_limiter = RateLimiter()
cache = Cache()
