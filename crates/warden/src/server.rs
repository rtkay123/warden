mod publish;
mod routes;
pub use routes::metrics::metrics_app;

use axum::Router;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};

#[cfg(feature = "redoc")]
use utoipa_redoc::Servable;
#[cfg(feature = "scalar")]
use utoipa_scalar::Servable as _;

use crate::{server::routes::ApiDoc, state::AppHandle};

pub fn router(state: AppHandle) -> Router {
    let (router, _api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health_check))
        .nest("/api", routes::processor::router(state.clone()))
        .split_for_parts();

    #[cfg(feature = "swagger")]
    let router = router.merge(
        utoipa_swagger_ui::SwaggerUi::new("/swagger-ui")
            .url("/api-docs/swaggerdoc.json", _api.clone()),
    );

    #[cfg(feature = "redoc")]
    let router = router.merge(utoipa_redoc::Redoc::with_url("/redoc", _api.clone()));

    #[cfg(feature = "rapidoc")]
    let router = router.merge(
        utoipa_rapidoc::RapiDoc::with_openapi("/api-docs/rapidoc.json", _api.clone())
            .path("/rapidoc"),
    );

    #[cfg(feature = "scalar")]
    let router = router.merge(utoipa_scalar::Scalar::with_url("/scalar", _api));

    warden_middleware::apply(router)
}

/// Get health of the API.
#[utoipa::path(
    method(get),
    path = "/",
    responses(
        (status = OK, description = "Success", body = str, content_type = "text/plain")
    )
)]
pub async fn health_check() -> impl axum::response::IntoResponse {
    let name = env!("CARGO_PKG_NAME");
    let ver = env!("CARGO_PKG_VERSION");

    format!("{name} v{ver} is live")
}

#[cfg(test)]
pub(crate) fn test_config() -> warden_stack::Configuration {
    use warden_stack::Configuration;

    let config_path = "warden.toml";

    let config = config::Config::builder()
        .add_source(config::File::new(config_path, config::FileFormat::Toml))
        .build()
        .unwrap();

    config.try_deserialize::<Configuration>().unwrap()
}

#[cfg(test)]
pub(crate) fn generate_id() -> String {
    let id = uuid::Uuid::new_v4().to_string();
    id.replace("-", "")
}

#[cfg(test)]
pub(crate) fn test_pacs008() -> warden_core::iso20022::pacs008::Pacs008Document {
    let msg_id = generate_id();
    let cre_dt_tm = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap();
    let end_to_end_id = generate_id();

    let debtor_fsp = "fsp001";
    let creditor_fsp = "fsp002";

    let ccy = "XTS";

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

    serde_json::from_value(v).unwrap()
}
