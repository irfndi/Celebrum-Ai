import unittest
from unittest.mock import MagicMock, AsyncMock, patch, PropertyMock
from sqlalchemy import create_engine
from sqlalchemy.orm import Session, sessionmaker
from src.meetsmatch.models import User, Interaction, Base
from src.meetsmatch.matching import Matcher
from datetime import datetime


class TestMatcher(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self):
        # Create an in-memory SQLite database for testing
        self.engine = create_engine("sqlite:///:memory:", echo=False)
        Base.metadata.create_all(self.engine)

        # Create a session factory with the test engine
        self.session_factory = sessionmaker(bind=self.engine)

        # Create a mock session
        self.session = MagicMock(spec=Session)
        self.session.bind = self.engine

        # Create the matcher with the mock session factory
        self.matcher = Matcher(session_factory=self.session_factory)

        # Setup common query chain mocks
        self.query_mock = MagicMock()
        self.filter_mock = MagicMock()
        self.session.query = MagicMock(return_value=self.query_mock)
        self.query_mock.filter = MagicMock(return_value=self.filter_mock)

        # Setup common session method mocks
        self.session.add = MagicMock()
        self.session.commit = MagicMock()
        self.session.rollback = MagicMock()
        self.session.close = MagicMock()

        # Override the session factory to return our mock session
        self.session_factory = MagicMock(return_value=self.session)
        self.matcher.session_factory = self.session_factory

    async def test_get_potential_matches(self):
        # Create a user and some potential matches
        user = User(
            telegram_id=1001,
            username="testuser",
            age=30,
            gender="Male",
            location="New York",
            interests='["hiking", "reading"]',
            is_active=True,
        )
        match1 = User(
            telegram_id=1002,
            username="match1",
            age=32,
            gender="Female",
            location="New York",
            interests='["reading", "music"]',
            is_active=True,
        )
        match2 = User(
            telegram_id=1003,
            username="match2",
            age=28,
            gender="Female",
            location="Los Angeles",
            interests='["hiking", "movies"]',
            is_active=True,
        )

        # Create some past interactions
        interaction1 = Interaction(
            user_id=user.id,
            target_user_id=4,  # Different user not in potential matches
            interaction_type="like",
            timestamp=datetime.utcnow(),
        )

        # Setup mocks for matching logic
        with patch.multiple(
            self.matcher,
            get_recent_interactions=AsyncMock(return_value=[interaction1]),
            get_country_from_city=MagicMock(return_value="United States"),
            get_shared_interests=MagicMock(return_value=["reading"]),
        ):
            # Mock is_profile_complete for all users
            type(user).is_profile_complete = PropertyMock(return_value=True)
            type(match1).is_profile_complete = PropertyMock(return_value=True)
            type(match2).is_profile_complete = PropertyMock(return_value=True)

            # Mock the users query to return potential matches
            self.filter_mock.all.return_value = [match1, match2]

            # Call the get_potential_matches method
            matches = await self.matcher.get_potential_matches(user)

            # Assert that the correct matches are returned
            self.assertEqual(len(matches), 2)
            self.assertIn(match1, matches)
            self.assertIn(match2, matches)

    async def test_get_potential_matches_no_matches(self):
        # Create a user with no potential matches
        user = User(
            telegram_id=1001,
            username="testuser",
            age=30,
            gender="Male",
            location="New York",
            interests='["hiking", "reading"]',
            is_active=True,
        )

        # Setup mocks for matching logic
        with patch.multiple(
            self.matcher,
            get_recent_interactions=AsyncMock(return_value=[]),
            get_country_from_city=MagicMock(return_value="United States"),
            get_shared_interests=MagicMock(return_value=[]),
        ):
            # Mock is_profile_complete
            type(user).is_profile_complete = PropertyMock(return_value=True)

            # Mock the users query to return empty list
            self.filter_mock.all.return_value = []

            # Call the get_potential_matches method
            matches = await self.matcher.get_potential_matches(user)

            # Assert that no matches are returned
            self.assertEqual(len(matches), 0)
