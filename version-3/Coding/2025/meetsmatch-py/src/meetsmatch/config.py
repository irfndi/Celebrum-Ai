import os


class Config:
    # Telegram Bot Configuration
    BOT_TOKEN = os.getenv("BOT_TOKEN", "YOUR_BOT_TOKEN")

    # Database Configuration
    DATABASE_URL = os.getenv(
        "DATABASE_URL",
        "postgresql://meetsmatch_user:securepassword@localhost:5432/meetsmatch",
    )
    POOL_SIZE = int(os.getenv("DB_POOL_SIZE", 5))
    MAX_OVERFLOW = int(os.getenv("DB_MAX_OVERFLOW", 10))

    # S3 Storage Configuration
    S3_ENDPOINT = os.getenv("S3_ENDPOINT")
    S3_ACCESS_KEY = os.getenv("S3_ACCESS_KEY")
    S3_SECRET_KEY = os.getenv("S3_SECRET_KEY")
    S3_BUCKET = os.getenv("S3_BUCKET", "meetsmatch-media")
    S3_REGION = os.getenv("S3_REGION", "us-east-1")

    # Media Configuration
    MAX_MEDIA_SIZE = 5 * 1024 * 1024  # 5MB
    MAX_MEDIA_FILES = 5
    MEDIA_RETENTION_DAYS = 180

    # Matching Configuration
    MATCH_AGE_RANGE = 4
    MATCH_LOCATION_RADIUS_KM = 50

    # Security Configuration
    SECRET_KEY = os.getenv("SECRET_KEY", "your-secret-key-here")
