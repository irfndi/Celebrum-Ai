# Telegram Bot Match – Feature Details

all the command only can be used direct on bot & with /start command, otherwise it will be ignored.

## /start Command

### Check Profile:
- If profile is incomplete, continue registration from the last step.
- If profile is complete, show "Welcome back..." and display profiles that have liked the user.
- If no likes are available, show the Main Menu:
    - `/match` – Start matching process.
    - `/profile` – View or update profile.
    - `/configuration` – Change language or delete account.

## Feature Details

### 1. Like & Dislike Mechanism

#### Profile Likes:
- Users can see profiles that have liked them (name, age, media, interests, location, bio).
- Users can like back or dislike the profile.

#### Match:
- If both users like each other, share Telegram usernames (as links).
- Save interaction data for matched profiles.

#### Dislike:
- If a user dislikes a profile, save the interaction data and make the profile available for matching again to the same user within 72 hours.

### 2. Multi-Language System

#### Language Setup:
- Set language during profile creation/registration.
- Language can be changed later via `/configuration` in the main menu.

#### Persistence:
- Ensure all bot commands and interactions are displayed in the selected language.

### 3. Registration Mechanism

#### Profile Completion Check:
- Users must complete all registration steps before using the bot.
- If incomplete, continue from the last step.

#### Profile Fields:
- **Name:** Free-form input (no first/last name requirement).
- **Gender:** Male or Female.
- **Age:** 12–100 years.
- **Bio:** Max 120 characters.
- **Location:**
    - Share location via Telegram or manually type the city.
    - Automatically match city with country in the database.
- **Interests:**
    - Flexible manual input with robust matching for typos, synonyms, and case sensitivity (e.g., "music" vs "MUSIC").
    - Optional field.
- **Media:**
    - Max 5 media files (images/videos).
    - Minimum 1 media file required.

### 4. Profile Mechanism

#### Profile Updates:
- Users can update their profile after registration via `/profile` command.
- Changes are saved and reflected immediately.

### 5. /start Mechanism

#### First Interaction:
- Check and set the user's language.
- Guide the user through registration steps if incomplete.
- If profile is complete, display profiles that have liked the user.
- If no likes are available, show the Main Menu.

### 6. Matching Mechanism

#### Matching Criteria:
- **Age:** Match within a range of -4 to +4 years of the user's age.
- **Location:** Match by city or nearby cities. If traffic is low, expand to country or nearby countries.
- **Interests:** Match based on shared or similar interests.

#### Matching Process:
- Users see other profiles and can use commands:
    - `/like` – Like the profile.
    - `/dislike` – Dislike the profile.
    - `/report` – Report the profile.

### 7. Report Mechanism

#### Manual Reporting:
- Save data on who reported whom for manual review.
- Ensure reported profiles are flagged for further action.

### 8. Delete Account Mechanism
- Users can delete their account and all associated data via `/configuration`.
- On database we mark soft delete and kee the data
- If users create with same telegram username, we still create new account other different ID

### 8. Media Mechanism
- Max 5 media files (images/videos) per profile with max size 5MB
- Have mechanism to store media in S3 compatible storage
- Have mechanism to delete media in 180 days after their change their media

## Additional Enhancements

### Profile Status:
- Add a profile status (e.g., "Active," "Inactive," "Banned") to manage user activity.
- Allow users to temporarily deactivate their profile via `/configuration`.

### Matching Algorithm Improvements:
- Introduce a scoring system for better match quality (e.g., higher weight for shared interests).
- Allow users to set preferences for age range, location, and interests.

### User Notifications:
- Notify users when they receive a new like or match.
- Send reminders to complete their profile if incomplete.

### Profile Verification:
- Add optional profile verification (e.g., photo ID) to increase trust and authenticity.

### Analytics Dashboard:
- Track user interactions, matches, and reports for insights and improvements.

## Example Flow

1. User sends `/start`.
2. Bot checks profile completion:
     - If incomplete, continue registration.
     - If complete, show profiles that have liked the user.
3. User interacts with profiles using `/like`, `/dislike`, or `/report`.
4. If a match occurs, share Telegram usernames.
5. User can update profile or change settings via `/profile` and `/configuration`.