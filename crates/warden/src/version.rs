use std::collections::HashMap;

use axum::{
    RequestPartsExt,
    extract::{FromRequestParts, Path},
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use utoipa::ToSchema;

#[derive(Debug, ToSchema)]
pub enum Version {
    V0,
}

impl<S> FromRequestParts<S> for Version
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let params: Path<HashMap<String, String>> =
            parts.extract().await.map_err(IntoResponse::into_response)?;

        let version = params
            .get("version")
            .ok_or_else(|| (StatusCode::NOT_FOUND, "version param missing").into_response())?;

        match version.as_str() {
            "v0" => Ok(Version::V0),
            _ => Err((StatusCode::NOT_FOUND, "unknown version").into_response()),
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use sqlx::PgPool;
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};
    use tower::ServiceExt;
    use warden_stack::cache::RedisManager;

    use crate::{
        server::{self, generate_id, test_config},
        state::{AppState, Services},
    };

    #[sqlx::test]
    async fn invalid_version(pool: PgPool) {
        let config = test_config();

        let cache = RedisManager::new(&config.cache).await.unwrap();
        let client = async_nats::connect(&config.nats.hosts[0]).await.unwrap();
        let jetstream = async_nats::jetstream::new(client);

        let state = AppState::create(
            Services {
                postgres: pool,
                cache,
                jetstream,
            },
            &test_config(),
        )
        .await
        .unwrap();
        let app = server::router(state);

        let ccy = "XTS";

        let msg_id = generate_id();
        let cre_dt_tm = OffsetDateTime::now_utc().format(&Rfc3339).unwrap();

        let debtor_fsp = "fsp001";
        let creditor_fsp = "fsp002";

        let end_to_end_id = generate_id();

        let v = serde_json::json!({
              "f_i_to_f_i_cstmr_cdt_trf": {
                "grp_hdr": {
                  "msg_id": msg_id,
                  "cre_dt_tm": cre_dt_tm,
                  "nb_of_txs": "CLRG",
                  "sttlm_inf": {
                    "sttlm_mtd": 1
                  }
                },
                "splmtry_data": [],
                "cdt_trf_tx_inf": [
                  {
                    "pmt_id": {
                      "instr_id": generate_id(),
                      "end_to_end_id": end_to_end_id
                    },
                    "intr_bk_sttlm_amt": {
                      "value": 294.3,
                      "ccy": ccy,
                    },
                    "instd_amt": {
                      "value": 294.3,
                      "ccy": ccy
                    },
                    "xchg_rate": 1,
                    "chrg_br": 1,
                    "chrgs_inf": [
                      {
                        "amt": {
                          "value": 0,
                          "ccy": ccy
                        },
                        "agt": {
                          "fin_instn_id": {
                            "clr_sys_mmb_id": {
                              "mmb_id": debtor_fsp,
                            }
                          }
                        }
                      }
                    ],
                    "initg_pty": {
                      "nm": "April Blake Grant",
                      "id": {
                        "org_id": {
                          "othr": []
                        },
                        "prvt_id": {
                          "dt_and_plc_of_birth": {
                            "birth_dt": "1968-02-01",
                            "city_of_birth": "Unknown",
                            "ctry_of_birth": "ZZ"
                          },
                          "othr": [
                            {
                              "id": "+27730975224",
                              "schme_nm": {
                                "prtry": "MSISDN",
                                "cd": "cd-value"
                              }
                            }
                          ]
                        }
                      },
                      "ctct_dtls": {
                        "mob_nb": "+27-730975224",
                        "othr": []
                      }
                    },
                    "dbtr": {
                      "nm": "April Blake Grant",
                      "id": {
                        "org_id": {
                          "othr": []
                        },
                        "prvt_id": {
                          "dt_and_plc_of_birth": {
                            "birth_dt": "2000-07-23",
                            "city_of_birth": "Unknown",
                            "ctry_of_birth": "ZZ"
                          },
                          "othr": [
                            {
                              "id": generate_id(),
                              "schme_nm": {
                                "prtry": "EID",
                                "cd": "cd-value"
                              }
                            }
                          ]
                        }
                      },
                      "ctct_dtls": {
                        "mob_nb": "+27-730975224",
                        "othr": []
                      }
                    },
                    "dbtr_acct": {
                      "id": {
                        "i_b_a_n": "value",
                        "othr": {
                          "id": generate_id(),
                          "schme_nm": {
                            "prtry": "MSISDN",
                            "cd": "value"
                          }
                        }
                      },
                      "nm": "April Grant"
                    },
                    "dbtr_agt": {
                      "fin_instn_id": {
                        "clr_sys_mmb_id": {
                          "mmb_id": debtor_fsp,
                        }
                      }
                    },
                    "cdtr_agt": {
                      "fin_instn_id": {
                        "clr_sys_mmb_id": {
                          "mmb_id": creditor_fsp,
                        }
                      }
                    },
                    "cdtr": {
                      "nm": "Felicia Easton Quill",
                      "id": {
                        "org_id": {
                          "othr": []
                        },
                        "prvt_id": {
                          "dt_and_plc_of_birth": {
                            "birth_dt": "1935-05-08",
                            "city_of_birth": "Unknown",
                            "ctry_of_birth": "ZZ"
                          },
                          "othr": [
                            {
                              "id": generate_id(),
                              "schme_nm": {
                                "prtry": "EID",
                                "cd": ""
                              }
                            }
                          ]
                        }
                      },
                      "ctct_dtls": {
                        "mob_nb": "+27-707650428",
                        "othr": []
                      }
                    },
                    "cdtr_acct": {
                      "id": {
                        "i_b_a_n": "",
                        "othr": {
                          "id": generate_id(),
                          "schme_nm": {
                            "prtry": "MSISDN",
                            "cd": "acc"
                          }
                        }
                      },
                      "nm": "Felicia Quill"
                    },
                    "purp": {
                      "cd": "MP2P",
                      "prtry": ""
                    },
                    "rgltry_rptg": [
                      {
                        "dtls": [
                          {
                            "tp": "BALANCE OF PAYMENTS",
                            "cd": "100",
                            "inf": []
                          }
                        ]
                      }
                    ],
                    "rmt_inf": {
                      "ustrd": [],
                      "strd": []
                    },
                    "splmtry_data": [
                      {
                        "envlp": {
                          "doc": {
                            "xprtn": "2021-11-30T10:38:56.000Z",
                            "initg_pty": {
                              "glctn": {
                                "lat": "-3.1609",
                                "long": "38.3588"
                              }
                            }
                          }
                        }
                      }
                    ],
                    "instr_for_cdtr_agt": [],
                    "instr_for_nxt_agt": [],
                    "rltd_rmt_inf": []
                  }
                ]
              }
        });
        let body = serde_json::to_vec(&v).unwrap();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .uri("/api/v99/pacs008")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
