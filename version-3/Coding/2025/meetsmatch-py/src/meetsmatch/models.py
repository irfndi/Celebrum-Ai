from sqlalchemy import (
    create_engine,
    Column,
    Integer,
    String,
    Boolean,
    DateTime,
    ForeignKey,
)
from sqlalchemy.orm import sessionmaker, relationship, DeclarativeBase, scoped_session
from datetime import datetime
import logging
import json

logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger(__name__)


class Base(DeclarativeBase):
    pass


engine = create_engine("sqlite:///meetsmatch.db")
Session = scoped_session(sessionmaker(bind=engine))


class Interaction(Base):
    __tablename__ = "interactions"

    id = Column(Integer, primary_key=True)
    user_id = Column(Integer, ForeignKey("users.id"))
    target_user_id = Column(Integer, ForeignKey("users.id"))
    interaction_type = Column(String)  # 'like' or 'dislike'
    timestamp = Column(DateTime, default=datetime.utcnow)
    created_at = Column(DateTime, default=datetime.utcnow)

    user = relationship("User", foreign_keys=[user_id])
    target_user = relationship("User", foreign_keys=[target_user_id])


class User(Base):
    __tablename__ = "users"

    interactions = relationship(
        "Interaction", foreign_keys="Interaction.user_id", back_populates="user"
    )
    received_interactions = relationship(
        "Interaction",
        foreign_keys="Interaction.target_user_id",
        back_populates="target_user",
    )

    id = Column(Integer, primary_key=True)
    telegram_id = Column(Integer, unique=True, nullable=False)
    username = Column(String)
    name = Column(String)
    gender = Column(String)
    preferred_gender = Column(String)
    age = Column(Integer)
    bio = Column(String, default=None)
    location = Column(String)
    interests = Column(String)
    media_urls = Column(String)  # JSON array of URLs
    language = Column(String, default="en")
    is_active = Column(Boolean, default=True)
    is_deleted = Column(Boolean, default=False)
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)

    def __init__(
        self,
        telegram_id,
        username=None,
        name=None,
        gender=None,
        age=None,
        bio=None,
        location=None,
        interests=None,
        media_urls=None,
        language="en",
        is_active=True,
        created_at=None,
        updated_at=None,
    ):
        self.telegram_id = telegram_id
        self.username = username
        self.name = name
        self.gender = gender
        self.age = age
        self.bio = bio
        self.location = location
        self.interests = interests
        self.media_urls = media_urls
        self.language = language
        self.is_active = is_active
        self.created_at = created_at or datetime.utcnow()
        self.updated_at = updated_at or datetime.utcnow()

    # Media relationship
    media_files = relationship("MediaFile", back_populates="user")

    @property
    def location_data(self):
        """Get location data as a dictionary with lat/lon."""
        if not self.location:
            return None
        try:
            return json.loads(self.location)
        except json.JSONDecodeError:
            # If location is a string, return just city/country
            return {"city": self.location, "country": None}

    @location_data.setter
    def location_data(self, value):
        """Set location data, ensuring it's stored as JSON."""
        if isinstance(value, str):
            # If string provided, store as is for backward compatibility
            self.location = value
        else:
            # Store dictionary as JSON string
            self.location = json.dumps(value)

    @property
    def is_profile_complete(self):
        """Check if user profile is complete with all required fields."""
        def is_valid_string(s):
            """Validate that a string is not empty or just whitespace."""
            return bool(s and str(s).strip())
            
        def is_valid_gender(s):
            """Validate gender string.
            
            Args:
                s: String to validate
                
            Returns:
                bool: True if gender is valid, False otherwise
                
            Valid genders are 'male' or 'female' (case-insensitive).
            Empty strings, None, or other values are invalid.
            """
            logger = logging.getLogger(__name__)
            logger.debug(f"Validating gender: {s!r}")
            logger.debug(f"Type: {type(s)}")
            
            # Check if None
            if s is None:
                logger.debug("Gender is None")
                return False
                
            # Check if not a string
            if not isinstance(s, str):
                logger.debug("Gender is not a string")
                return False
                
            # Strip whitespace first
            s = s.strip()
            
            # Check if empty after stripping
            if not s:
                logger.debug("Gender is empty after stripping")
                return False
            logger.debug(f"Stripped gender: {s!r}")
                
            # Validate against allowed values
            is_valid = s.lower() in {"male", "female"}
            logger.debug(f"Gender validation result: {is_valid}")
            return is_valid
            
        def is_valid_interests(s):
            """Validate interests JSON string.
            
            Args:
                s: String to validate
                
            Returns:
                bool: True if interests is valid, False otherwise
                
            Valid interests must be a non-empty JSON array of non-empty strings.
            """
            if not s:
                return False
            try:
                interests = json.loads(s)
                return bool(interests and isinstance(interests, list) and 
                          all(isinstance(i, str) and i.strip() for i in interests))
            except (json.JSONDecodeError, TypeError, AttributeError):
                return False

        # Check all required fields
        required_fields = {
            "username": bool(self.username and len(str(self.username).strip()) >= 5),
            "name": is_valid_string(self.name),
            "gender": is_valid_gender(self.gender),
            "age": bool(self.age and isinstance(self.age, int) and 12 <= self.age <= 100),
            "bio": bool(self.bio and 10 <= len(str(self.bio).strip()) <= 120),
            "location": is_valid_string(self.location),
            "interests": is_valid_interests(self.interests),
            "media_files": bool(self.media_files and len(self.media_files) >= 1)
        }
        
        # For debugging
        logger = logging.getLogger(__name__)
        logger.debug("Checking profile completeness")
        for field, is_valid in required_fields.items():
            logger.debug(f"Field {field}: {is_valid}")
            if not is_valid:
                logger.debug(f"Field {field} is invalid")
                logger.debug(f"Value: {getattr(self, field, None)!r}")
                if field == "gender" and self.gender:
                    logger.debug(f"Gender stripped: '{str(self.gender).strip()}'")
                elif field == "media_files":
                    logger.debug(f"Media files count: {len(self.media_files) if self.media_files else 0}")
                
        logger.debug(f"Profile completeness result: {all(required_fields.values())}")
        return all(required_fields.values())

    def to_dict(self):
        return {
            "id": self.id,
            "username": self.username,
            "name": self.name,
            "gender": self.gender,
            "age": self.age,
            "bio": self.bio,
            "location": self.location,
            "interests": self.interests,
            "media_urls": self.media_urls,
            "language": self.language,
            "is_active": self.is_active,
        }


class MediaFile(Base):
    __tablename__ = "media_files"

    id = Column(Integer, primary_key=True)
    user_id = Column(Integer, ForeignKey("users.id"))
    file_type = Column(String)  # 'image' or 'video'
    file_id = Column(String)  # Telegram file_id
    s3_key = Column(String)  # S3 storage key
    size_bytes = Column(Integer)
    is_active = Column(Boolean, default=True)
    is_deleted = Column(Boolean, default=False)
    created_at = Column(DateTime, default=datetime.utcnow)
    deleted_at = Column(DateTime)

    # Relationship
    user = relationship("User", back_populates="media_files")


class Report(Base):
    __tablename__ = "reports"

    id = Column(Integer, primary_key=True)
    reporter_id = Column(Integer, ForeignKey("users.id"))
    reported_id = Column(Integer, ForeignKey("users.id"))
    reason = Column(String)
    created_at = Column(DateTime, default=datetime.utcnow)

    reporter = relationship("User", foreign_keys=[reporter_id])
    reported = relationship("User", foreign_keys=[reported_id])


# Create tables
Base.metadata.create_all(engine)
