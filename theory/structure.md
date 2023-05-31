# Structure

## Board - full game data
- whose turn is it
- what are the *o* & *x* placements

## AI - compute best move for a given board
- returns chosen move

## GameManager - handle the flow
- query move from AI and player
- store current board
- update current board

## App
- visualize board for player
- send input from player to game manager