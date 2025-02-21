import { BoardType, PlayerType } from "../components/types.tsx";

export default class Game {
    socket: WebSocket;
    player: PlayerType;
    playerTurn?: PlayerType;
    board: BoardType = [0, 0, 0, 0, 0, 0, 0, 0, 0]; // initialize game board
    constructor(socket: WebSocket, player: PlayerType) {
        this.socket = socket;
		this.player = player;
    }
    handleMove(position: number, player: string) {
        if (this.playerTurn != player) {
            console.log(`tried to take turn when playerTurn: ${this.playerTurn} != ${player}`);
            return;
        }
        this.board[position] = this.playerTurn;
    }
}