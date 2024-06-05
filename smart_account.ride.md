# Smart account for deployer

https://testnet.welocal.dev/node-0/utils/script/compile

```
curl -X POST "https://testnet.welocal.dev/node-0/utils/script/compile" -H "accept: application/json" -H "Content-Type: application/json" -d "<<SCRIPT>>"
```

### RIDE Script

```
{-# STDLIB_VERSION 2 #-}
{-# CONTENT_TYPE EXPRESSION #-}
{-# SCRIPT_TYPE ACCOUNT #-}

let publicKey0 = base58'yMQKms5WvLvobErygwGjByEuNuebLMGXHndfVDsjMVD';
let publicKey1 = base58'BN9meJdnaezqtUK7iGhWC9a6TvgU51ESc69wT8x7AnN8';
let publicKey2 = base58'ENV5mvh5GsDNHhqwYt1BzxfZew1M3rRRzXub5vaGxY3C';
let publicKey3 = base58'nobcGCfJ1ZG1J6g8T9dRLoUnBCgQ6DM5H8Hy78sAmSN';
let publicKey4 = base58'Hv2T217jAFbgjXiqrz2CKQkbFH9CJc9dFAgwcQmi3Q83';

match tx {
    case t:SetScriptTransaction | UpdateContractTransaction =>
        let verify0 = if sigVerify(tx.bodyBytes, tx.proofs[0], publicKey0) then 1 else 0;
        let verify1 = if sigVerify(tx.bodyBytes, tx.proofs[1], publicKey1) then 1 else 0;
        let verify2 = if sigVerify(tx.bodyBytes, tx.proofs[2], publicKey2) then 1 else 0;
        let verify3 = if sigVerify(tx.bodyBytes, tx.proofs[3], publicKey3) then 1 else 0;
        let verify4 = if sigVerify(tx.bodyBytes, tx.proofs[4], publicKey4) then 1 else 0;
    (verify0 + verify1 + verify2 + verify3 + verify4) >= 3
    case _ => sigVerify(tx.bodyBytes, tx.proofs[0], tx.senderPublicKey)
}
```

### Compiled bytecode

```
{
"script": "base64:AQQAAAAKcHVibGljS2V5MAEAAAAgDm+YFSkwCqf1zsqINC/ueLTOTkYSFUUJx04G/LD4HGQEAAAACnB1YmxpY0tleTEBAAAAIJn/+3E9wiA+lNqb4n3vdwyek7lILDnWveY9WRt49TIFBAAAAApwdWJsaWNLZXkyAQAAACDGqOE7feP+8+Rqb1H3nz1v4+cfTrHQI9iFBKMedo0DQwQAAAAKcHVibGljS2V5MwEAAAAgC7vSzcu+7xT0FBi4+XAe+GuemnmeOv8tZsHMp/92T1cEAAAACnB1YmxpY0tleTQBAAAAIPtQaV9ISzJgnXFGVBL/rLZoxL8SqKN8Jx4NhrwM4dJ0BAAAAAckbWF0Y2gwBQAAAAJ0eAMDCQAAAQAAAAIFAAAAByRtYXRjaDACAAAAGVVwZGF0ZUNvbnRyYWN0VHJhbnNhY3Rpb24GCQAAAQAAAAIFAAAAByRtYXRjaDACAAAAFFNldFNjcmlwdFRyYW5zYWN0aW9uBAAAAAF0BQAAAAckbWF0Y2gwBAAAAAd2ZXJpZnkwAwkAAfQAAAADCAUAAAACdHgAAAAJYm9keUJ5dGVzCQABkQAAAAIIBQAAAAJ0eAAAAAZwcm9vZnMAAAAAAAAAAAAFAAAACnB1YmxpY0tleTAAAAAAAAAAAAEAAAAAAAAAAAAEAAAAB3ZlcmlmeTEDCQAB9AAAAAMIBQAAAAJ0eAAAAAlib2R5Qnl0ZXMJAAGRAAAAAggFAAAAAnR4AAAABnByb29mcwAAAAAAAAAAAQUAAAAKcHVibGljS2V5MQAAAAAAAAAAAQAAAAAAAAAAAAQAAAAHdmVyaWZ5MgMJAAH0AAAAAwgFAAAAAnR4AAAACWJvZHlCeXRlcwkAAZEAAAACCAUAAAACdHgAAAAGcHJvb2ZzAAAAAAAAAAACBQAAAApwdWJsaWNLZXkyAAAAAAAAAAABAAAAAAAAAAAABAAAAAd2ZXJpZnkzAwkAAfQAAAADCAUAAAACdHgAAAAJYm9keUJ5dGVzCQABkQAAAAIIBQAAAAJ0eAAAAAZwcm9vZnMAAAAAAAAAAAMFAAAACnB1YmxpY0tleTMAAAAAAAAAAAEAAAAAAAAAAAAEAAAAB3ZlcmlmeTQDCQAB9AAAAAMIBQAAAAJ0eAAAAAlib2R5Qnl0ZXMJAAGRAAAAAggFAAAAAnR4AAAABnByb29mcwAAAAAAAAAABAUAAAAKcHVibGljS2V5NAAAAAAAAAAAAQAAAAAAAAAAAAkAAGcAAAACCQAAZAAAAAIJAABkAAAAAgkAAGQAAAACCQAAZAAAAAIFAAAAB3ZlcmlmeTAFAAAAB3ZlcmlmeTEFAAAAB3ZlcmlmeTIFAAAAB3ZlcmlmeTMFAAAAB3ZlcmlmeTQAAAAAAAAAAAMJAAH0AAAAAwgFAAAAAnR4AAAACWJvZHlCeXRlcwkAAZEAAAACCAUAAAACdHgAAAAGcHJvb2ZzAAAAAAAAAAAACAUAAAACdHgAAAAPc2VuZGVyUHVibGljS2V5R2KcFA==",
"complexity": 668
}
```
