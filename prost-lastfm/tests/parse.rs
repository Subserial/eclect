use prost_lastfm::user;

const GET_INFO_RAW: &[u8] = br##"
{
    "user": {
        "name": "TestUser",
        "age": "0",
        "subscriber": "1",
        "realname": "",
        "bootstrap": "0",
        "playcount": "1000",
        "artist_count": "2000",
        "playlists": "0",
        "track_count": "3000",
        "album_count": "4000",
        "image":[
            {
                "size": "small",
                "#text": "https://example.com/small.png"
            },
            {
                "size": "medium",
                "#text": "https://example.com/medium.png"
            },
            {
                "size": "large",
                "#text": "https://example.com/large.png"
            },
            {
                "size": "extralarge",
                "#text": "https://example.com/extralarge.png"
            }
        ],
        "registered": {
            "unixtime": "1350000000",
            "#text": 1350000000
        },
        "country": "United States",
        "gender": "n",
        "url": "https://www.last.fm/user/TestUser",
        "type": "subscriber"
    }
}
"##;

const GET_RECENT_TRACKS_RAW: &[u8] = br##"
{
    "recenttracks": {
        "track": [
            {
                "artist": {
                    "mbid": "69158f97-4c07-4c4e-baf8-4e4ab1ed666e",
                    "#text": "Boards of Canada"
                },
                "streamable": "0",
                "image": [
                    {
                        "size": "small",
                        "#text": "https://lastfm.freetls.fastly.net/i/u/34s/77b2419ede333b1b20ab565305bd8039.png"
                    },
                    {
                        "size": "medium",
                        "#text": "https://lastfm.freetls.fastly.net/i/u/64s/77b2419ede333b1b20ab565305bd8039.png"
                    },
                    {
                        "size": "large",
                        "#text": "https://lastfm.freetls.fastly.net/i/u/174s/77b2419ede333b1b20ab565305bd8039.png"
                    },
                    {
                        "size": "extralarge",
                        "#text": "https://lastfm.freetls.fastly.net/i/u/300x300/77b2419ede333b1b20ab565305bd8039.png"
                    }
                ],
                "mbid": "4131e57f-4826-4c0d-89aa-5205e0fee918",
                "album": {
                    "mbid": "21f8abb6-d3ce-4047-9b53-c3ccc4783eef",
                    "#text": "The Campfire Headphase"
                },
                "name": "Into the Rainbow Vein",
                "@attr": {
                    "nowplaying": "true"
                },
                "url": "https://www.last.fm/music/Boards+of+Canada/_/Into+the+Rainbow+Vein"
            },
            {
                "artist": {
                    "mbid": "69158f97-4c07-4c4e-baf8-4e4ab1ed666e",
                    "#text": "Boards of Canada"
                },
                "streamable": "0",
                "image": [
                    {
                        "size": "small",
                        "#text": "https://lastfm.freetls.fastly.net/i/u/34s/77b2419ede333b1b20ab565305bd8039.png"
                    },
                    {
                        "size": "medium",
                        "#text": "https://lastfm.freetls.fastly.net/i/u/64s/77b2419ede333b1b20ab565305bd8039.png"
                    },
                    {
                        "size": "large",
                        "#text": "https://lastfm.freetls.fastly.net/i/u/174s/77b2419ede333b1b20ab565305bd8039.png"
                    },
                    {
                        "size": "extralarge",
                        "#text": "https://lastfm.freetls.fastly.net/i/u/300x300/77b2419ede333b1b20ab565305bd8039.png"
                    }
                ],
                "mbid": "4131e57f-4826-4c0d-89aa-5205e0fee918",
                "album": {
                    "mbid": "21f8abb6-d3ce-4047-9b53-c3ccc4783eef",
                    "#text": "The Campfire Headphase"
                },
                "name": "Into the Rainbow Vein",
                "url": "https://www.last.fm/music/Boards+of+Canada/_/Into+the+Rainbow+Vein",
                "date": {
                    "uts": "1732873371",
                    "#text": "29 Nov 2024, 09:42"
                }
            }
        ],
        "@attr": {
            "user": "TestUser",
            "totalPages": "10000",
            "page": "1",
            "perPage": "1",
            "total": "10000"
        }
    }
}
"##;

#[test]
fn parse() -> Result<(), String> {
    match serde_json::from_slice::<user::GetInfoResponse>(GET_INFO_RAW) {
        Err(err) => return Err(err.to_string()),
        Ok(resp) => {
            println!("{:?}", resp);
        }
    }
    match serde_json::from_slice::<user::GetRecentTracksResponse>(GET_RECENT_TRACKS_RAW) {
        Err(err) => return Err(err.to_string()),
        Ok(resp) => {
            println!("{:?}", resp);
        }
    }
    Ok(())
}
