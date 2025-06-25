import { UserQueries } from '@celebrum-ai/db';
import type { User, NewUser } from '@celebrum-ai/db/schema';

export class UserService {
  private userQueries: UserQueries;

  constructor(db: any) {
    this.userQueries = new UserQueries(db);
  }

  async findUserByTelegramId(telegramId: string): Promise<User | undefined> {
    return this.userQueries.findByTelegramId(telegramId);
  }

  async createUser(userData: Partial<NewUser>): Promise<User> {
    const newUser: NewUser = {
      telegramId: userData.telegramId!,
      firstName: userData.firstName,
      lastName: userData.lastName,
      username: userData.username,
      status: 'active',
      role: 'free',
      ...userData,
    };
    return this.userQueries.create(newUser);
  }

  async updateUser(id: number, updates: Partial<NewUser>): Promise<User | undefined> {
    return this.userQueries.update(id, updates);
  }
}