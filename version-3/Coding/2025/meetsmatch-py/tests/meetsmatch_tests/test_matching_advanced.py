import pytest
from unittest.mock import MagicMock, patch
from src.meetsmatch.models import Interaction
from src.meetsmatch.matching import Matcher

class TestMatchingAdvanced:
    # [TODO] Add test_location_expansion() - City->Country->Region matching
    # [TODO] Add test_interest_synonym_matching() - "music" vs "MUSIC" vs "songs"  
    # [TODO] Add test_dislike_cooldown_period() - 72-hour rematch prevention

    @pytest.fixture
    def setup(self):
        """Setup fixture that runs before each test"""
        self.session = MagicMock()
        self.session_factory = MagicMock(return_value=self.session)
        self.matcher = Matcher(session_factory=self.session_factory)
        
        # Configure query chain mocks
        self.filter_mock = MagicMock()
        self.query_mock = MagicMock()
        self.query_mock.filter.return_value = self.filter_mock
        self.query_mock.filter_by.return_value = self.filter_mock
        self.session.query.return_value = self.query_mock
        
        # Mock geolocator with proper syntax
        self.geolocator_patch = patch(
            "src.meetsmatch.matching.Nominatim",
            return_value=MagicMock(
                geocode=MagicMock(
                    return_value=MagicMock(
                        raw={"address": {"country": "United States"}},
                        latitude=40.7128,
                        longitude=-74.0060
                    )
                )
            )
        )
        self.mock_geolocator = self.geolocator_patch.start()
        
        yield  # Run test
        
        # Teardown
        self.geolocator_patch.stop()

    @pytest.mark.asyncio
    async def test_like_profile_saves_interaction(self, setup):
        """Test that liking a profile saves an interaction."""
        user = MagicMock(id=1)
        target_user = MagicMock(id=2)
        
        # Configure mocks
        self.filter_mock.first.return_value = None
        
        # Call the function
        await self.matcher.like_profile(user, target_user)
        
        # Verify interaction was saved
        self.session.add.assert_called_once()
        saved_interaction = self.session.add.call_args[0][0]
        assert isinstance(saved_interaction, Interaction)
        assert saved_interaction.user_id == user.id
        assert saved_interaction.target_user_id == target_user.id
        assert saved_interaction.interaction_type == "like"
