from geopy.geocoders import Nominatim
from geopy.exc import GeocoderTimedOut
import json
from difflib import SequenceMatcher
from .models import User

class InterestMatcher:
    def __init__(self):
        # Initialize word embeddings and similarity model
        self.synonyms = {
            "programming": ["coding", "development", "software"],
            "music": ["singing", "songs", "musical"],
            "movies": ["films", "cinema", "television", "tv"],
            "sports": ["athletics", "exercise", "fitness"],
            "reading": ["books", "literature", "novels"],
            "cooking": ["baking", "culinary", "food"],
            "photography": ["photos", "pictures", "cameras"],
            "travel": ["traveling", "travelling", "trips"],
            "gaming": ["games", "videogames", "esports"],
            "art": ["drawing", "painting", "crafts"],
        }

        # Build reverse mapping
        self.interest_map = {}
        for main, synonyms in self.synonyms.items():
            self.interest_map[main] = main
            for syn in synonyms:
                self.interest_map[syn] = main

    def normalize_interest(self, interest: str) -> str:
        """Normalize an interest by finding its main category or closest match."""
        interest = interest.lower().strip()

        # Direct match in map
        if interest in self.interest_map:
            return self.interest_map[interest]

        # Find closest match using fuzzy matching
        best_match = None
        best_ratio = 0

        for known in self.interest_map.keys():
            ratio = SequenceMatcher(None, interest, known).ratio()
            if ratio > 0.8 and ratio > best_ratio:  # 80% similarity threshold
                best_match = known
                best_ratio = ratio

        return self.interest_map[best_match] if best_match else interest

    def get_matching_score(self, user_interests, match_interests):
        """
        Calculate interest matching score between two users.
        """
        if not user_interests or not match_interests:
            return 0.0

        # Normalize interests
        norm1 = [self.normalize_interest(i) for i in user_interests]
        norm2 = [self.normalize_interest(i) for i in match_interests]

        # Check if we have enough shared interests
        shared_interests = [
            i
            for i in user_interests
            if any(
                self.normalize_interest(i) == self.normalize_interest(m)
                for m in match_interests
            )
        ]

        # Calculate score based on common interests and total interests
        return len(shared_interests) / max(len(norm1), len(norm2))




class LocationMatcher:
    def __init__(self):
        self.geolocator = Nominatim(user_agent="meetsmatch_app_v1")
        self.location_cache = {}

    def get_matching_score(self, user_location, match_location):
        """
        Calculate location matching score between two users.
        """
        try:
            # Get coordinates for both locations
            user_coords = self._get_location_coords(user_location)
            match_coords = self._get_location_coords(match_location)

            if not user_coords or not match_coords:
                return 0

            # Calculate distance-based score
            distance = self._calculate_distance(user_coords, match_coords)
            return self._distance_to_score(distance)

        except GeocoderTimedOut:
            return 0

    def _get_location_coords(self, location):
        """Get coordinates for a location with caching."""
        if location in self.location_cache:
            return self.location_cache[location]

        try:
            geocoded = self.geolocator.geocode(location)
            if geocoded:
                coords = (geocoded.latitude, geocoded.longitude)
                self.location_cache[location] = coords
                return coords
            return None
        except GeocoderTimedOut:
            return None

    def get_location_info(self, location_str, retries=3):
        """Get geolocation details with retry logic."""
        try:
            location = self.geolocator.geocode(location_str, exactly_one=True)
            if not location:
                return {}
            
            # Extract city from structured address components
            address_data = location.raw.get('address', {})
            city = address_data.get('city') or \
                   address_data.get('town') or \
                   address_data.get('village') or \
                   "Unknown"
            country = address_data.get('country', 'Unknown')
            
            return {
                "lat": location.latitude,
                "lon": location.longitude,
                "address": location.address,
                "city": city,
                "country": country,
                "raw": location_str
            }
        except GeocoderTimedOut:
            if retries > 0:
                return self.get_location_info(location_str, retries-1)
            return {}

    def calculate_distance(self, loc1, loc2):
        """Calculate distance between two locations in kilometers using Haversine."""
        from math import radians, sin, cos, sqrt, atan2

        # Earth radius in kilometers
        R = 6371.0

        lat1, lon1 = radians(loc1["lat"]), radians(loc1["lon"])
        lat2, lon2 = radians(loc2["lat"]), radians(loc2["lon"])

        dlon = lon2 - lon1
        dlat = lat2 - lat1

        a = sin(dlat/2)**2 + cos(lat1) * cos(lat2) * sin(dlon/2)**2
        c = 2 * atan2(sqrt(a), sqrt(1-a))

        return R * c

    def get_location_score(self, loc1, loc2, max_distance=100):
        """Calculate normalized location match score (0-1) based on distance."""
        if not loc1 or not loc2:
            return 0.0
            
        distance = self.calculate_distance(loc1, loc2)
        return max(0.0, 1.0 - (distance / max_distance))


class EnhancedMatcher:
    def __init__(self, session=None):
        self.session = session
        self.interest_matcher = InterestMatcher()
        self.location_matcher = LocationMatcher()

    def find_matches(self, user: User, limit: int = 10,
                    max_distance: int = 50, min_shared_interests: int = 1):
        """
        Get potential matches for a user using enhanced matching algorithm.

        Args:
            user (User): The user to find matches for
            limit (int): Maximum number of matches to return
            max_distance (int): Maximum distance in km for location matching
            min_shared_interests (int): Minimum number of shared interests

        Returns:
            List[User]: List of potential matches, sorted by compatibility
        """
        session = self.session
        try:
            # Get user's location data
            user_loc = user.location_data
            if not user_loc:
                return []

            # Get user's interests
            try:
                user_interests = json.loads(user.interests) if user.interests else []
            except json.JSONDecodeError:
                user_interests = []

            # Query for potential matches
            query = session.query(User).filter(
                User.id != user.id,
                User.is_active == True,
                User.is_deleted == False,
                User.gender.isnot(None),
                User.gender != user.gender  # Only match with opposite gender
            ).all()

            # Get location info if needed
            if isinstance(user_loc, str):
                user_loc = self.location_matcher.get_location_info(user_loc)
            elif isinstance(user_loc, dict) and ("lat" not in user_loc or "lon" not in user_loc):
                user_loc = self.location_matcher.get_location_info(user_loc.get("city", ""))

            if not user_loc or "lat" not in user_loc or "lon" not in user_loc:
                return []

            matches = []
            for match in query:
                # Get match location
                match_loc = match.location_data
                if not match_loc:
                    continue

                if isinstance(match_loc, str):
                    match_loc = self.location_matcher.get_location_info(match_loc)
                elif isinstance(match_loc, dict) and ("lat" not in match_loc or "lon" not in match_loc):
                    match_loc = self.location_matcher.get_location_info(match_loc.get("city", ""))

                # Skip if location info couldn't be retrieved
                if not match_loc or "lat" not in match_loc or "lon" not in match_loc:
                    continue

                # Calculate distance
                distance = self.location_matcher.calculate_distance(user_loc, match_loc)
                if distance > max_distance:
                    continue

                # Calculate location score based on distance
                location_score = self.location_matcher.get_location_score(user_loc, match_loc, max_distance)
                if location_score == 0:  # Too far away
                    continue

                # Get match interests
                try:
                    match_interests = json.loads(match.interests) if match.interests else []
                except json.JSONDecodeError:
                    match_interests = []

                # Calculate interest score
                interest_score = self._calculate_interest_score(user_interests, match_interests)
                if len(set(user_interests) & set(match_interests)) < min_shared_interests:
                    continue

                # Calculate final score
                final_score = self._calculate_final_score(interest_score, location_score)

                matches.append((match, final_score))

            # Sort by score and return top matches
            matches.sort(key=lambda x: x[1], reverse=True)
            return [match for match, _ in matches[:limit]]

        except Exception as e:
            print(f"Error in find_matches: {e}")
            return []

    def _calculate_interest_score(self, user_interests, match_interests):
        """Calculate interest matching score between two users."""
        return self.interest_matcher.get_matching_score(user_interests, match_interests)

    def _calculate_location_score(self, user_loc, match_loc):
        """Calculate location matching score between two users."""
        if not user_loc or not match_loc:
            return 0.0
        
        try:
            return self.location_matcher.get_location_score(user_loc, match_loc)
        except (KeyError, ValueError):
            return 0.0

    def _calculate_final_score(self, interest_score, location_score):
        """Calculate final matching score using weighted sum."""
        interest_weight = 0.7
        location_weight = 0.3
        total_weight = interest_weight + location_weight
        return (
            interest_score * interest_weight + location_score * location_weight
        ) / total_weight


enhanced_matcher = EnhancedMatcher()
