import unittest
from unittest.mock import MagicMock, patch
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker
from sqlalchemy.pool import StaticPool

from meetsmatch.models import Base, User


class TestBase(unittest.TestCase):
    """Base class for all test cases."""

    def setUp(self):
        """Set up test case."""
        # Create in-memory database
        self.engine = create_engine(
            "sqlite:///:memory:",
            connect_args={"check_same_thread": False},
            poolclass=StaticPool,
        )

        # Create all tables
        Base.metadata.create_all(self.engine)

        # Create session factory
        self.Session = sessionmaker(
            bind=self.engine,
            expire_on_commit=False
        )

        # Create session for this test
        self.session = self.Session()

        # Create test user
        self.user = User(
            telegram_id=1234567890,
            username="test_user",
            name="Test User",
            age=25,
            gender="male",
            location="New York",
            bio="Test bio",
            interests="coding,testing",
        )
        self.session.add(self.user)
        self.session.commit()

        # Mock S3 client
        self.mock_s3 = MagicMock()
        self.s3_patcher = patch("meetsmatch.media.boto3.client")
        mock_s3_client = self.s3_patcher.start()
        mock_s3_client.return_value = self.mock_s3

    def tearDown(self):
        """Clean up after test."""
        self.session.close()
        Base.metadata.drop_all(self.engine)
        self.s3_patcher.stop()
