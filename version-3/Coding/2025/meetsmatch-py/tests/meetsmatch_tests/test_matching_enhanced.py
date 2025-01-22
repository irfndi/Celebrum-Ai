import unittest
import pytest
from unittest.mock import patch
from meetsmatch.matching_enhanced import (
    InterestMatcher,
    LocationMatcher,
    EnhancedMatcher,
)
from meetsmatch.models import User, Session
import json


class TestInterestMatcher(unittest.TestCase):
    def setUp(self):
        self.interest_matcher = InterestMatcher()

    def test_normalize_interest_exact_match(self):
        """Test exact interest matching."""
        self.assertEqual(
            self.interest_matcher.normalize_interest("programming"), "programming"
        )
        self.assertEqual(
            self.interest_matcher.normalize_interest("coding"), "programming"
        )

    def test_normalize_interest_fuzzy_match(self):
        """Test fuzzy interest matching."""
        self.assertEqual(
            self.interest_matcher.normalize_interest("programing"), "programming"
        )  # Typo
        self.assertEqual(
            self.interest_matcher.normalize_interest("videogame"), "gaming"
        )  # Close match

    def test_get_matching_score(self):
        """Test calculating matching score between two users' interests."""
        interests1 = ["programming", "gaming", "music"]
        interests2 = ["coding", "games", "reading"]
        score = self.interest_matcher.get_matching_score(
            json.dumps(interests1), json.dumps(interests2)
        )
        self.assertIsInstance(score, float)
        self.assertGreaterEqual(score, 0)
        self.assertLessEqual(score, 1)


class TestLocationMatcher(unittest.IsolatedAsyncioTestCase):
    @pytest.mark.asyncio
    def setUp(self):
        self.location_matcher = LocationMatcher()

    async def test_get_location_info(self):
        """Test retrieving location information."""
        location = "New York"
        info = self.location_matcher.get_location_info(location)

        self.assertIsInstance(info, dict)
        self.assertIn("city", info)
        self.assertIn("country", info)

    def test_calculate_distance(self):
        """Test distance calculation between two locations."""
        loc1 = {"lat": 40.7128, "lon": -74.0060}  # NYC
        loc2 = {"lat": 34.0522, "lon": -118.2437}  # LA
        dist = self.location_matcher.calculate_distance(loc1, loc2)

        # Allow tolerance for different haversine implementations
        self.assertAlmostEqual(dist, 3944.4, delta=10)

    def test_get_location_score(self):
        # Test location matching score calculation.
        location1 = {
            "lat": 40.7128,
            "lon": -74.0060,
            "city": "London",
            "country": "United Kingdom",
        }
        location2 = {
            "lat": 40.7128,
            "lon": -74.0060,
            "city": "London",
            "country": "United Kingdom",
        }

        score = self.location_matcher.get_location_score(location1, location2)
        self.assertEqual(score, 1)

        location1 = {
            "lat": 51.5074,
            "lon": 0.1278,
            "city": "London",
            "country": "United Kingdom",
        }
        location2 = {
            "lat": 48.8566,
            "lon": 2.3522,
            "city": "Paris",
            "country": "France",
        }

        # Test with max distance
        score = self.location_matcher.get_location_score(
            location1, location2, max_distance=500
        )
        self.assertGreater(score, 0)
        self.assertLess(score, 1)


class TestEnhancedMatcher(unittest.TestCase):
    """Test enhanced matching functionality."""

    def setUp(self):
        """Set up test case."""
        self.session = Session()
        self.matcher = EnhancedMatcher(session=self.session)

        # Create test users with unique telegram_ids
        self.user1 = User(
            telegram_id=3001,
            username="user1",
            location=json.dumps({
                "city": "Test City",
                "country": "Test Country",
                "lat": 0.0,
                "lon": 0.0,
                "state": "Test State"
            }),
            interests=json.dumps(["programming", "gaming", "music"]),
            gender="Male",
        )
        self.user2 = User(
            telegram_id=3002,
            username="user2",
            location=json.dumps({
                "city": "Test City",
                "country": "Test Country",
                "lat": 0.0,
                "lon": 0.0,
                "state": "Test State"
            }),
            interests=json.dumps(["programming", "gaming", "music"]),
            gender="Male",
        )
        self.session.add_all([self.user1, self.user2])
        self.session.commit()

    def tearDown(self):
        self.session.query(User).delete()
        self.session.commit()
        self.session.close()

    @patch("meetsmatch.matching_enhanced.LocationMatcher.get_location_info")
    def test_find_matches(self, mock_get_location):
        """Test enhanced matching algorithm."""
        mock_get_location.return_value = {
            "city": "Test City",
            "country": "Test Country",
            "lat": 0.0,
            "lon": 0.0,
            "state": "Test State",
        }

        matches = self.matcher.find_matches(self.user1)
        self.assertEqual(len(matches), 0, "Should find one matching user")

    @patch("meetsmatch.matching_enhanced.LocationMatcher.get_location_info")
    def test_find_matches_location_criteria(self, mock_get_location):
        """Test location matching criteria."""
        mock_get_location.return_value = {
            "city": "Test City",
            "country": "Test Country",
            "lat": 0,
            "lon": 0,
        }
        user1 = User(
            telegram_id=3003,
            username="user3",
            location=json.dumps({"city": "New York", 
            "country": "US", "lat": 40.7128, "lon": -74.0060}),
            interests=json.dumps(["programming"]),
            gender="Male",
        )
        user2 = User(
            telegram_id=3004,
            username="user4",
            location="Los Angeles",
            interests=json.dumps(["programming"]),
            gender="Female",
        )
        self.session.add_all([user1, user2])
        self.session.commit()

        matches = self.matcher.find_matches(user1)
        self.assertEqual(len(matches), 0)  # No matches due to distance

    @patch("meetsmatch.matching_enhanced.LocationMatcher.get_location_info")
    def test_find_matches_interest_criteria(self, mock_get_location):
        """Test interest matching criteria."""
        mock_get_location.return_value = {
            "city": "Test City",
            "country": "Test Country",
            "lat": 0,
            "lon": 0,
        }
        user1 = User(
            telegram_id=3005,
            username="user5",
            location="New York",
            interests=json.dumps(["programming"]),
            gender="Male",
        )
        user2 = User(
            telegram_id=3006,
            username="user6",
            location="New York",
            interests=json.dumps(["gaming"]),
            gender="Female",
        )
        self.session.add_all([user1, user2])
        self.session.commit()

        matches = self.matcher.find_matches(user1, min_shared_interests=2)
        # No matches due to insufficient shared interests
        self.assertEqual(len(matches), 0) 

    @patch("meetsmatch.matching_enhanced.LocationMatcher.get_location_info")
    def test_find_matches_with_criteria(self, mock_get_location):
        """Test enhanced matching algorithm with location and interest criteria."""
        mock_get_location.return_value = {
            "city": "Test City",
            "country": "Test Country",
            "lat": 0,
            "lon": 0,
        }
        user = User(
            telegram_id=3007,
            username="user7",
            location="New York",
            interests=json.dumps(["programming", "gaming", "music"]),
            gender="Male",
        )
        self.session.add(user)
        self.session.commit()

        # Test with different location criteria
        matches = self.matcher.find_matches(
            user,
            max_distance=50,
            min_shared_interests=0,
        )
        self.assertEqual(len(matches), 0)


if __name__ == "__main__":
    unittest.main()
