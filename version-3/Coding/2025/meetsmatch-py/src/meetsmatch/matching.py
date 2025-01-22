from .models import User, Session, Interaction, Report
import json
from geopy.geocoders import Nominatim
from datetime import datetime, timedelta
from typing import List, Optional
import logging

logger = logging.getLogger(__name__)


class Matcher:
    def __init__(self, session_factory=None):
        self.geolocator = Nominatim(user_agent="meetsmatch_bot")
        self.session_factory = session_factory or Session

    def get_country_from_city(self, city: str) -> Optional[str]:
        """Get the country for a given city using geocoding."""
        try:
            location = self.geolocator.geocode(city)
            if location and "address" in location.raw:
                return location.raw["address"].get("country")
            return None
        except Exception as e:
            logger.error(f"Error getting country for city {city}: {str(e)}")
            return None

    async def get_potential_matches(self, user: User) -> List[User]:
        """
        Get potential matches for a user.

        Args:
            user (User): The user to find matches for

        Returns:
            List[User]: List of potential matches, sorted by compatibility
        """
        session = self.session_factory()
        try:
            # Get all active users except the current user
            users = (
                session.query(User)
                .filter(User.is_active == True, User.id != user.id)  # noqa: E712
                .all()
            )

            # Filter out users that have been interacted with in the last 72 hours
            recent_interactions = await self.get_recent_interactions(user)
            interacted_user_ids = {i.target_user_id for i in recent_interactions}
            users = [u for u in users if u.id not in interacted_user_ids]

            matches = []
            for potential_match in users:
                if self.is_match(user, potential_match):
                    matches.append(potential_match)

            # Sort matches by number of shared interests
            matches.sort(
                key=lambda m: len(self.get_shared_interests(user, m)), reverse=True
            )

            return matches
        finally:
            session.close()

    async def get_recent_interactions(self, user: User) -> List[Interaction]:
        """
        Get recent interactions for a user.

        Args:
            user (User): The user to find interactions for

        Returns:
            List[Interaction]: List of recent interactions
        """
        session = self.session_factory()
        try:
            recent_interactions = (
                session.query(Interaction)
                .filter(
                    Interaction.user_id == user.id,
                    Interaction.created_at >= datetime.utcnow() - timedelta(hours=72),
                )
                .all()
            )
            return recent_interactions
        finally:
            session.close()

    def is_match(self, user: User, potential_match: User) -> bool:
        """
        Check if two users are a potential match.

        Args:
            user (User): The first user
            potential_match (User): The potential match

        Returns:
            bool: True if users match, False otherwise
        """
        print(
            f"Checking match between user {user.id} and "
            f"potential_match {potential_match.id}"
        )
        print(f"User location: {user.location}")
        print(f"Potential match location: {potential_match.location}")

        # Check if both users have complete profiles
        if not user.is_profile_complete or not potential_match.is_profile_complete:
            print(
                "Profile not complete - "
                f"user: {user.is_profile_complete}, "
                f"potential_match: {potential_match.is_profile_complete}"
            )
            return False

        # Age check (within 4 years)
        if not (user.age - 4 <= potential_match.age <= user.age + 4):
            print(
                "Age mismatch - "
                f"user: {user.age}, "
                f"potential_match: {potential_match.age}"
            )
            return False

        # Location check (same country)
        user_country = self.get_country_from_city(user.location)
        match_country = self.get_country_from_city(potential_match.location)
        print(f"User country: {user_country}")
        print(f"Potential match country: {match_country}")
        if not user_country or not match_country or user_country != match_country:
            print(
                "Country mismatch - "
                f"user: {user_country}, "
                f"potential_match: {match_country}"
            )
            return False

        # Interest check (at least one shared interest)
        shared_interests = self.get_shared_interests(user, potential_match)
        if not shared_interests:
            print("No shared interests")
            return False

        print("Match found!")
        return True

    def get_shared_interests(self, user1: User, user2: User) -> List[str]:
        """
        Get shared interests between two users.

        Args:
            user1 (User): First user
            user2 (User): Second user

        Returns:
            List[str]: List of shared interests
        """
        try:
            interests1 = set(json.loads(user1.interests))
            interests2 = set(json.loads(user2.interests))
            return list(interests1.intersection(interests2))
        except (json.JSONDecodeError, TypeError):
            return []

    async def get_matches(self, user: User) -> List[User]:
        """
        Get a list of matches for the user.

        Args:
            user (User): The user for whom to find matches.

        Returns:
            List[User]: A list of users who are matches for the given user.
        """
        return await self.get_potential_matches(user)

    async def like_profile(self, user: User, target_user: User) -> None:
        """
        Record a like from one user to another.

        Args:
            user (User): The user who is liking.
            target_user (User): The user being liked.
        """
        session = self.session_factory()
        try:
            interaction = Interaction(
                user_id=user.id,
                target_user_id=target_user.id,
                interaction_type="like",
                created_at=datetime.utcnow(),
            )
            session.add(interaction)
            session.commit()

            # Check if the target user has also liked the user
            reciprocal_like = (
                session.query(Interaction)
                .filter(
                    Interaction.user_id == target_user.id,
                    Interaction.target_user_id == user.id,
                    Interaction.interaction_type == "like",
                )
                .first()
            )

            if reciprocal_like:
                # It's a match!
                await self.match_users(user, target_user)
        finally:
            session.close()

    async def dislike_profile(self, user: User, target_user: User) -> None:
        """
        Record a dislike from one user to another.

        Args:
            user (User): The user who is disliking.
            target_user (User): The user being disliked.
        """
        session = self.session_factory()
        try:
            interaction = Interaction(
                user_id=user.id,
                target_user_id=target_user.id,
                interaction_type="dislike",
                created_at=datetime.utcnow(),
            )
            session.add(interaction)
            session.commit()
        finally:
            session.close()

    async def report_profile(self, user: User, target_user: User, reason: str) -> None:
        """
        Report a user profile.

        Args:
            user (User): The user who is reporting.
            target_user (User): The user being reported.
            reason (str): The reason for the report.
        """
        session = self.session_factory()
        try:
            report = Report(
                reporter_id=user.id,
                reported_id=target_user.id,
                reason=reason,
                created_at=datetime.utcnow(),
            )
            session.add(report)
            session.commit()
        finally:
            session.close()

    async def match_users(self, user1: User, user2: User) -> bool:
        """
        Handle a match between two users.

        Args:
            user1 (User): The first user.
            user2 (User): The second user.

        Returns:
            bool: True if the match was successful, False otherwise.
        """
        session = self.session_factory()
        try:
            # Share Telegram usernames
            user1.matched_user_id = user2.telegram_id
            user2.matched_user_id = user1.telegram_id

            # Create match interaction
            match_interaction = Interaction(
                user_id=user1.id,
                target_user_id=user2.id,
                interaction_type="match",
                created_at=datetime.utcnow(),
            )

            # Save the match data
            session.add(user1)
            session.add(user2)
            session.add(match_interaction)
            session.commit()

            return True
        finally:
            session.close()
