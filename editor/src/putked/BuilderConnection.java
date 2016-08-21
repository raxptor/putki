package putked;

import java.io.DataOutputStream;
import java.net.Socket;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.Map.Entry;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.locks.Condition;
import java.util.concurrent.locks.Lock;
import java.util.concurrent.locks.ReentrantLock;

public class BuilderConnection implements Runnable
{
	static class Change
	{
		public DataObject object;
		public int version;
	}

	static HashMap<String, Change> m_changedObjects = new HashMap<String, Change>();
	static final Lock m_changeLock = new ReentrantLock();
	static boolean m_acceptChanges = false;
	static final Condition m_hasChanges = m_changeLock.newCondition();

	public static void onObjectChanged(DataObject object)
	{
		DataObject actual = object.getRoot();

		Change c = new Change();
		c.object = actual;
		String path = actual.getPath();
		m_changeLock.lock();
		if (!m_acceptChanges)
		{
			m_changeLock.unlock();
			return;
		}

		try
		{
			if (m_changedObjects.containsKey(path))
			{
				c.version = m_changedObjects.get(path).version + 1;
			}
			else
			{
				c.version = 1;
			}
			m_changedObjects.put(path, c);
			m_hasChanges.signal();
		}
		catch (Exception e)
		{
			System.out.println(e.toString());
		}
		finally
		{
			m_changeLock.unlock();
		}
	}

	static Thread s_thr = null;

	public static void start()
	{
		m_changeLock.lock();
		m_acceptChanges = true;
		m_changedObjects.clear();
		m_changeLock.unlock();

		s_thr = new Thread(new BuilderConnection());
		s_thr.setDaemon(true);
		s_thr.start();
	}

	public void run()
	{
		do
		{
			Socket sock = null;
			try
			{
				sock = new Socket("localhost", 5555);
				DataOutputStream output = new DataOutputStream(sock.getOutputStream());
				output.writeBytes("<ready>\n");

				HashMap<String, Integer> changesSent = new HashMap<String, Integer>();

				while (true)
				{
					ArrayList<DataObject> toSend = new ArrayList<DataObject>();
					m_changeLock.lock();
					try
					{
						boolean changes;
						do
						{
							changes = false;
							for(Entry<String, Change> entry : m_changedObjects.entrySet())
							{
							    String path = entry.getKey();
							    int version = entry.getValue().version;
							    if (!changesSent.containsKey(path) || changesSent.get(path) != version)
							    {
							    	toSend.add(entry.getValue().object);
							    	changesSent.put(path,  version);
							    	changes = true;
							    }
							}
							if (!changes)
							{
								m_hasChanges.await(500, TimeUnit.MILLISECONDS);
								output.writeBytes("<keepalive>\n");
							}
						}
						while (!changes);
					}
					catch (Exception e)
					{
						break;
					}
					finally
					{
						m_changeLock.unlock();
					}

					for(DataObject obj : toSend)
					{
						output.writeBytes(obj.getPath() + "\n");
						output.writeBytes(Main.s_dataWriter.writeAsset(obj, false).toString());
						output.writeBytes("<end>\n");
					}
				}
			}
			catch (Exception e)
			{

			}
			finally
			{
				if (sock != null)
				{
					try
					{
						sock.close();
						Thread.sleep(500);
					}
					catch (Exception e)
					{

					}
				}
			}
		}
		while (true);
	}
}

