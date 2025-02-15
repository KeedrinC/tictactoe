type Repeat<T, N extends number, R extends T[] = []> = R['length'] extends N ? R : Repeat<T, N, [...R, T]>;

type GameType = {
    playerType: string | undefined,
    board: Repeat<string | 0, 9>
};

export type { GameType };