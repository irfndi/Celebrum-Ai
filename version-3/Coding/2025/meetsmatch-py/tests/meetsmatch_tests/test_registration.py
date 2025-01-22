from random import randint
from src.meetsmatch.models import User, MediaFile, Session
import logging
from datetime import datetime
from .test_base import TestBase


logging.basicConfig(
    level=logging.DEBUG,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
    stream=logging.sys.stdout,
)


def create_media_file(user, is_deleted=False):
    """Helper to create a MediaFile instance for testing."""
    media = MediaFile(
        user=user,
        file_type="image/jpeg",
        file_id="test_file_id",
        s3_key="test_s3_key",
        size_bytes=1000,
        is_deleted=is_deleted,
        created_at=datetime.utcnow(),
    )
    return media


class TestRegistration(TestBase):
    """Test registration functionality."""

    def setUp(self):
        """Set up test case."""
        super().setUp()

        # Clear any SQLAlchemy session state
        Session.remove()

        # Create a fresh session for this test
        self.session = Session()

        # Create a fresh User instance with no attributes set
        self.user = User(telegram_id=randint(100000, 999999))
        self.session.add(self.user)
        self.session.commit()

        # Reset all attributes to None
        self.user.name = None
        self.user.gender = None
        self.user.age = None
        self.user.bio = None
        self.user.location = None
        self.user.interests = None
        self.user.media_files = []
        self.user.is_deleted = False
        self.user.is_banned = False
        self.user.is_active = True
        self.user.is_verified = False
        self.user.last_active = None
        self.user.created_at = None
        self.user.updated_at = None
        self.session.commit()

        # Create test media files
        media1 = MediaFile(
            user_id=self.user.id,
            file_type="photo",
            s3_key="users/3001/photo1.jpg",
            is_active=True
        )
        media2 = MediaFile(
            user_id=self.user.id,
            file_type="photo",
            s3_key="users/3001/photo2.jpg",
            is_active=True,
        )
        self.user.media_files.extend([media1, media2])
        self.session.commit()

    def tearDown(self):
        logger = logging.getLogger(__name__)
        logger.debug("\nTearing down test...")

        # Clear any cached properties
        if hasattr(self.user, "_is_profile_complete"):
            delattr(self.user, "_is_profile_complete")

        # Rollback any pending changes
        self.session.rollback()
        self.session.expire_all()

        # Delete all test users
        self.session.query(MediaFile).delete()
        self.session.query(User).delete()
        self.session.commit()

        # Clear session
        self.session.close()
        Session.remove()

        logger.debug("Teardown complete.")

    def test_complete_registration(self):
        """Test that all required fields are properly validated."""
        logger = logging.getLogger(__name__)
        logger.debug("\nStarting test_complete_registration...")

        # Set all fields to valid values
        self.user.username = "testuser"
        self.user.name = "Test User"
        self.user.gender = "Male"
        self.user.age = 30
        self.user.bio = "Test bio with valid content"
        self.user.location = "Test location"
        self.user.interests = '["coding","testing"]'
        self.session.commit()

        # Verify profile is complete with all fields set
        self.assertTrue(self.user.is_profile_complete)

    def test_incomplete_registration(self):
        """Test that missing required fields result in incomplete profile."""
        logger = logging.getLogger(__name__)
        logger.debug("\nStarting test_incomplete_registration...")

        # Test empty string values - FIXED COMMA SEPARATION
        string_fields = ["gender", "name", "bio", "location"]  # Already has commas
        for field in string_fields:
            logger.debug(f"\nTesting {field} being empty string...")

            # Clean up any existing test users and commit
            self.session.query(User).delete()
            self.session.commit()
            self.session.expire_all()

            # Create a new user instance with all required fields except the one being tested
            test_user = User(
                telegram_id=randint(100000, 999999),
                username="testuser",  # Required: at least 5 chars
                name="Test User" if field != "name" else "",
                gender="Male" if field != "gender" else "",
                age=25,  # Required: between 12 and 100
                bio="This is a valid test bio" if field != "bio" else "",  # Required: 10-120 chars
                location="Test City" if field != "location" else "",
                interests='["coding", "testing"]',  # Required: valid JSON array
            )
            self.session.add(test_user)
            self.session.commit()
            
            # Remove all media files and refresh relationships
            self.session.query(MediaFile).filter_by(user_id=test_user.id).delete()
            self.session.commit()
            self.session.expire(test_user, ['media_files'])  # Force reload media relationship
            self.session.refresh(test_user)
            logger.debug(f"  Set {field} to empty string, other fields are valid")

            # Check that profile is incomplete
            is_complete = test_user.is_profile_complete
            logger.debug(f"  is_profile_complete: {is_complete}")
            logger.debug(f"  {field} value: '{getattr(test_user, field)}'")
            msg = f"Profile should be incomplete when {field} is an empty string"
            self.assertFalse(is_complete, msg)

            # Clean up test user
            self.session.delete(test_user)
            self.session.commit()
            self.session.expire_all()

        # Test None values
        required_fields = ["gender", "name", "age", "bio", "location", "interests"]
        for field in required_fields:
            logger.debug(f"\nTesting {field} being None...")

            # Clean up any existing test users and commit
            self.session.query(User).delete()
            self.session.commit()
            self.session.expire_all()

            # Create a new user instance for each test case with a unique telegram_id
            test_user = User(
                telegram_id=randint(100000, 999999),
                username="testuser",  # Required: at least 5 chars
                name="Test User" if field != "name" else None,
                gender="Male" if field != "gender" else None,
                age=25 if field != "age" else None,
                bio="This is a valid test bio" if field != "bio" else None,
                location="Test City" if field != "location" else None,
                interests='["coding", "testing"]' if field != "interests" else None,
            )
            self.session.add(test_user)
            self.session.commit()

            # Add required media file
            media = MediaFile(
                user_id=test_user.id,
                file_type="photo",
                s3_key="media1.jpg",
                is_active=True
            )
            test_user.media_files.append(media)
            self.session.commit()
            logger.debug(f"  Set {field} to None, other fields are valid")

            # Check that profile is incomplete
            is_complete = test_user.is_profile_complete
            logger.debug(f"  is_profile_complete: {is_complete}")
            logger.debug(f"  {field} value: {getattr(test_user, field)!r}")
            msg = f"Profile should be incomplete when {field} is None"
            self.assertFalse(is_complete, msg)

            # Clean up test user and media
            self.session.delete(media)
            self.session.delete(test_user)
            self.session.commit()
            self.session.expire_all()

        # Test invalid age values
        logger.debug("\nTesting invalid age values...")
        invalid_ages = [0, -1, None]
        for age in invalid_ages:
            logger.debug(f"\nTesting age being {age!r}...")

            # Clean up any existing test users and commit
            self.session.query(User).delete()
            self.session.commit()
            self.session.expire_all()

            # Create a new user with all valid fields except age
            test_user = User(
                telegram_id=randint(100000, 999999),
                username="testuser",
                name="Test User",
                gender="Male",
                age=age,
                bio="This is a valid test bio",
                location="Test City",
                interests='["coding", "testing"]'
            )
            self.session.add(test_user)
            self.session.commit()

            # Add required media file
            media = MediaFile(
                user_id=test_user.id,
                file_type="photo",
                s3_key="media1.jpg",
                is_active=True
            )
            test_user.media_files.append(media)
            self.session.commit()

            # Check profile incomplete
            is_complete = test_user.is_profile_complete
            logger.debug(f"  is_profile_complete: {is_complete}")
            logger.debug(f"  age value: {test_user.age!r}")
            self.assertFalse(
                is_complete,
                f"Profile should be incomplete when age is {age!r}",
            )

            # Clean up test user and media
            self.session.delete(media)
            self.session.delete(test_user)
            self.session.commit()
            self.session.expire_all()
