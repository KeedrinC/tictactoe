type Repeat<T, N extends number, R extends T[] = []> = R['length'] extends N ? R : Repeat<T, N, [...R, T]>;

type BoardType = Repeat<"X" | "O" | 0, 9>;
type PlayerType = "X" | "O";

type Lobby = { code: string }
type Session = {
    access_token: string,
    nickname: string
}

export type { PlayerType, BoardType, Lobby, Session };