use error::Error;
use failure;

pub fn curl(url: &str) -> Result<Vec<u8>, failure::Error> {
    let mut data = Vec::new();

    let mut easy = curl::easy::Easy::new();
    with_url_context(url, easy.follow_location(true))?;
    with_url_context(url, easy.url(url))?;
    transfer(url, &mut easy, &mut data)?;

    let status_code = with_url_context(url, easy.response_code())?;
    if 200 <= status_code && status_code < 300 {
        Ok(data)
    } else {
        Err(Error::http(&format!(
            "received a bad HTTP status code ({}) when requesting {}",
            status_code, url
        ))
        .into())
    }
}

fn with_url_context<T, E>(url: &str, r: Result<T, E>) -> Result<T, impl failure::Fail>
where
    Result<T, E>: failure::ResultExt<T, E>,
{
    use failure::ResultExt;
    r.with_context(|_| format!("when requesting {}", url))
}

fn transfer(
    url: &str,
    easy: &mut curl::easy::Easy,
    data: &mut Vec<u8>,
) -> Result<(), failure::Error> {
    let mut transfer = easy.transfer();
    with_url_context(
        url,
        transfer.write_function(|part| {
            data.extend_from_slice(part);
            Ok(part.len())
        }),
    )?;
    with_url_context(url, transfer.perform())?;
    Ok(())
}
